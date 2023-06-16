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
    fn invoke(&self, module: &Self::Module<'_>, stdin: &[u8]) -> Result<Vec<u8>>;
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

        fn bind<P, R>(
            &self,
            module: &Self::Module<'_>,
            fn_name: &str,
        ) -> anyhow::Result<Box<dyn Fn(P) -> R>> {
            let f = module
                .find_function::<P, R>(fn_name)
                .expect("function to be found");

            Ok(Box::new(|p: P| f.call(p)))
        }
    }
}

#[cfg(feature = "native")]
mod wasmtime {
    use super::{Result, Wasm};
    use std::cell::RefCell;
    use wasi_common::pipe::{ReadPipe, WritePipe};
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
                .typed::<(), ()>(&store)?
                .call(&mut store, ())?;
            Ok(())
        }

        fn invoke(&self, module: &Self::Module<'_>, stdin: &[u8]) -> Result<Vec<u8>> {
            let stdin = ReadPipe::from(stdin);
            let stdout = WritePipe::new_in_memory();
            let stderr = WritePipe::new_in_memory();

            let cx = WasiCtxBuilder::new()
                .stdin(Box::new(stdin.clone()))
                .stdout(Box::new(stdout.clone()))
                .stderr(Box::new(stderr.clone()))
                .inherit_args()?
                .build();

            let mut linker = self.linker.borrow_mut();
            let mut store = Store::new(linker.engine(), cx);
            linker.module(&mut store, "", module)?;
            linker
                .get_default(&mut store, "")?
                .typed::<(), ()>(&store)?
                .call(&mut store, ())?;

            if let Ok(err) = stderr.try_into_inner() {
                let err_bytes = err.into_inner();
                let err_message =
                    std::string::String::from_utf8(err_bytes).map_err(anyhow::Error::msg)?;

                Err(anyhow::Error::msg(err_message))
            } else if let Ok(out) = stdout.try_into_inner() {
                Ok(out.into_inner())
            } else {
                Ok(vec![])
            }
        }
    }
}

#[cfg(feature = "web")]
mod web {
    use crate::{Result, Wasm};
    use wasmer::{Instance, Module, Store};
    use wasmer_wasi::{Pipe, WasiState, WasiStateBuilder};

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

        fn invoke(&self, module: &Self::Module<'_>, stdin: &[u8]) -> Result<Vec<u8>> {
            let stdin = Pipe::from(stdin);

            let mut env = WasiState::new("service")
                .stdin(stdin)
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

            let mut out = vec![];
            let mut state = env.state();
            let stdout = state.fs.stdout_mut().unwrap().as_mut().unwrap();
            stdout.read_to_end(&mut out).unwrap();

            Ok(out)
        }
    }
}
