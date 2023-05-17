#[cfg(feature = "embedded")]
pub use crate::wasm3::*;
#[cfg(feature = "native")]
pub use crate::wasmtime::*;
#[cfg(feature = "web")]
pub use crate::web::*;

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

#[cfg(feature = "web")]
mod web {
    use crate::{Result, Wasm};
    use wasmer::{Instance, Module, Store};
    use wasmer_wasi::WasiState;

    pub struct Runtime(Store);

    impl Wasm for Runtime {
        type Module<'a> = Module;

        fn with_defaults() -> Self {
            Runtime(Store::new())
        }

        fn load(&self, module: &[u8]) -> Result<Self::Module<'_>> {
            Module::from_binary(&self.0, module).map_err(anyhow::Error::msg)
        }

        fn run(&self, module: &Self::Module<'_>) -> Result<()> {
            let mut env = WasiState::new("service")
                .finalize()
                .map_err(anyhow::Error::msg)?;
            let imports = env.import_object(&module).map_err(anyhow::Error::msg)?;
            let _ = Instance::new(&module, &imports)
                .map_err(anyhow::Error::msg)?
                .exports
                .get_function("_start")
                .map_err(anyhow::Error::msg)?
                .call(&[])
                .map_err(anyhow::Error::msg)?;

            // let mut out = String::new();
            // let mut state = env.state();
            // let stdout = state.fs.stdout_mut().unwrap().as_mut().unwrap();
            // stdout.read_to_string(&mut out).unwrap();
            Ok(())
        }
    }
}
