#![feature(once_cell)]

#[cfg(feature = "wasm3")]
pub use crate::wasm3::*;
#[cfg(feature = "wasmtime")]
pub use crate::wasmtime::*;

use anyhow::Result;

pub trait Wasm {
    type Module<'a>: Module
    where
        Self: 'a;

    fn with_defaults() -> Self;
    fn load(&self, module: &[u8]) -> Result<Self::Module<'_>>;
}

pub trait Module {
    fn start(&self) -> Result<()>;
}

#[cfg(feature = "wasm3")]
mod wasm3 {
    use super::{Module, Result, Wasm};
    use wasm3;
    pub use wasm3::Runtime;

    impl Wasm for wasm3::Runtime {
        type Module<'a> = wasm3::Module<'a>;

        fn with_defaults() -> Self {
            Self::new(&wasm3::Environment::new().unwrap(), 1024).expect("enough memory")
        }

        fn load(&self, module: &[u8]) -> Result<Self::Module<'_>> {
            let mut m = self.parse_and_load_module(module).unwrap();
            m.link_wasi();
            Ok(m)
        }
    }

    impl<'a> Module for wasm3::Module<'a> {
        fn start(&self) -> anyhow::Result<()> {
            let start = self.find_function::<(), ()>("_start").expect("has start");
            start.call().unwrap();
            Ok(())
        }
    }
}

#[cfg(feature = "wasmtime")]
mod wasmtime {
    use super::{Module, Result, Wasm};
    use once_cell::sync::OnceCell;
    use wasmtime::{self, Engine, Linker};
    use wasmtime_wasi::{add_to_linker, WasiCtx};

    static LINKER: OnceCell<Linker<WasiCtx>> = OnceCell::new();

    pub type Runtime = Engine;

    impl Wasm for Engine {
        type Module<'a> = wasmtime::Module;

        fn with_defaults() -> Self {
            let rt = Default::default();
            let mut linker = Linker::new(&rt);
            add_to_linker(&mut linker, |c| c).unwrap();
            let _ = LINKER.set(linker);
            rt
        }

        fn load(&self, module: &[u8]) -> Result<Self::Module<'_>> {
            wasmtime::Module::from_binary(self, module)
        }
    }

    impl Module for wasmtime::Module {
        fn start(&self) -> Result<()> {
            let l = LINKER.get();
            l.module(&mut store, "", &self);
        }
    }
}
