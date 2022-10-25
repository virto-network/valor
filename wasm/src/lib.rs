#![feature(once_cell)]

#[cfg(feature = "wasm3")]
pub use crate::wasm3::*;
#[cfg(feature = "wasmtime")]
pub use crate::wasmtime::*;

use anyhow::Result;

pub trait Wasm {
    type Module<'a>
    where
        Self: 'a;

    fn with_defaults() -> Self;
    fn load(&self, module: &[u8]) -> Result<Self::Module<'_>>;
    fn run(&self, module: &Self::Module<'_>) -> Result<()>;
}

#[cfg(feature = "embedded")]
mod wasm3 {
    use super::{Result, Wasm};
    use wasm3;
    pub use wasm3::Runtime;

    const STACK_SIZE: u32 = 1024 * 2;

    impl Wasm for wasm3::Runtime {
        type Module<'a> = wasm3::Module<'a>;

        fn with_defaults() -> Self {
            Self::new(&wasm3::Environment::new().unwrap(), STACK_SIZE).expect("enough memory")
        }

        fn load(&self, module: &[u8]) -> Result<Self::Module<'_>> {
            let mut m = self.parse_and_load_module(module).unwrap();
            let _ = m.link_wasi();
            Ok(m)
        }

        fn run(&self, module: &Self::Module<'_>) -> anyhow::Result<()> {
            let start = module.find_function::<(), ()>("_start").expect("has start");
            start.call().unwrap();
            Ok(())
        }
    }
}

#[cfg(feature = "native")]
mod wasmtime {
    use super::{Result, Wasm};
    use std::cell::RefCell;
    use wasmtime::{self, Engine, Linker, Store};
    use wasmtime_wasi::{add_to_linker, WasiCtx, WasiCtxBuilder};

    pub struct Runtime {
        linker: RefCell<Linker<WasiCtx>>,
    }

    impl Wasm for Runtime {
        type Module<'a> = wasmtime::Module;

        fn with_defaults() -> Self {
            let engine = Engine::default();
            let mut linker = Linker::new(&engine);
            add_to_linker(&mut linker, |c| c).expect("");
            Runtime {
                linker: RefCell::new(linker),
            }
        }

        fn load(&self, module: &[u8]) -> Result<Self::Module<'_>> {
            let linker = self.linker.borrow();
            wasmtime::Module::from_binary(linker.engine(), module)
        }

        fn run(&self, module: &Self::Module<'_>) -> Result<()> {
            let cx = WasiCtxBuilder::new()
                .inherit_stdio()
                .inherit_args()?
                .build();
            let mut linker = self.linker.borrow_mut();
            let mut store = Store::new(linker.engine(), cx);
            linker.module(&mut store, "", module)?;
            linker
                .get_default(&mut store, "")?
                .typed::<(), (), _>(&store)?
                .call(&mut store, ())?;
            Ok(())
        }
    }
}
