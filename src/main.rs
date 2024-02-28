wasmtime::component::bindgen!({
    world: "test-runner",
    path: "./component/wit",
    with: {
        "wasi:io/error": wasmtime_wasi::preview2::bindings::io::error,
        "wasi:io/streams": wasmtime_wasi::preview2::bindings::io::streams,
        "wasi:io/poll": wasmtime_wasi::preview2::bindings::io::poll,
        "wasi:http/types": wasmtime_wasi_http::bindings::http::types,

    },
    async: true
});

#[tokio::main]
async fn main() {
    let component_binary_path = std::env::args()
        .nth(1)
        .expect("Binary must be invoked with a path to the test runner component binary");

    let mut runtime = Runtime::new();
    let component = runtime.component(component_binary_path).unwrap();
    let instance = runtime.instance(&component).await.unwrap();
    let mut runtime_builder = RuntimeBuilder::new(&mut runtime, &instance).await.unwrap();
    runtime_builder.build().await.unwrap();
}

struct Runtime {
    engine: wasmtime::Engine,
    store: wasmtime::Store<View>,
    linker: wasmtime::component::Linker<View>,
}

impl Runtime {
    fn new() -> Self {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true).async_support(true);
        let engine = wasmtime::Engine::new(&config).unwrap();
        let dir = cap_std::fs::Dir::open_ambient_dir(".", cap_std::ambient_authority()).unwrap();
        let dir_perms = wasmtime_wasi::preview2::DirPerms::all();
        let file_perms = wasmtime_wasi::preview2::FilePerms::all();
        let path = ".";
        let view = View {
            table: wasmtime::component::ResourceTable::default(),
            ctx: wasmtime_wasi::preview2::WasiCtxBuilder::new()
                .inherit_stdout()
                .preopened_dir(dir, dir_perms, file_perms, path)
                .build(),
            http_ctx: wasmtime_wasi_http::WasiHttpCtx,
        };
        let store = wasmtime::Store::new(&engine, view);
        let mut linker: wasmtime::component::Linker<View> =
            wasmtime::component::Linker::new(&engine);
        wasmtime_wasi::preview2::command::add_to_linker(&mut linker).unwrap();
        component::spin_test_runner::spin::add_to_linker(&mut linker, |x| x).unwrap();
        wasmtime_wasi_http::bindings::http::types::add_to_linker(&mut linker, |x| x).unwrap();
        Self {
            engine,
            store,
            linker,
        }
    }

    fn component(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<wasmtime::component::Component> {
        wasmtime::component::Component::from_file(&self.engine, path)
    }

    async fn instance(
        &mut self,
        component: &wasmtime::component::Component,
    ) -> anyhow::Result<Instance> {
        let inner = self
            .linker
            .instantiate_async(&mut self.store, component)
            .await?;
        Ok(Instance { inner })
    }
}

struct Instance {
    inner: wasmtime::component::Instance,
}

struct RuntimeBuilder<'a> {
    runtime: &'a mut Runtime,
    test_runner: TestRunner,
    inner: wasmtime::component::ResourceAny,
}

impl<'a> RuntimeBuilder<'a> {
    async fn new(runtime: &'a mut Runtime, instance: &Instance) -> anyhow::Result<Self> {
        let test_runner = TestRunner::new(&mut runtime.store, &instance.inner).unwrap();
        let inner = test_runner
            .component_spin_test_runner_runner()
            .runtime_builder()
            .call_constructor(&mut runtime.store)
            .await?;
        Ok(Self {
            runtime,
            inner,
            test_runner,
        })
    }

    async fn build(&mut self) -> anyhow::Result<()> {
        let g = self.test_runner.component_spin_test_runner_runner();
        g.runtime_builder()
            .call_build(&mut self.runtime.store, self.inner)
            .await?;
        Ok(())
    }
}

struct View {
    table: wasmtime::component::ResourceTable,
    ctx: wasmtime_wasi::preview2::WasiCtx,
    http_ctx: wasmtime_wasi_http::WasiHttpCtx,
}

impl wasmtime_wasi::preview2::WasiView for View {
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut wasmtime_wasi::preview2::WasiCtx {
        &mut self.ctx
    }
}

use component::spin_test_runner::spin::{HttpRequest, HttpResponse};

#[async_trait::async_trait]
impl component::spin_test_runner::spin::HostInstance for View {
    async fn create(
        &mut self,
    ) -> wasmtime::Result<
        Result<wasmtime::component::Resource<component::spin_test_runner::spin::Instance>, String>,
    > {
        todo!("Instance::create was called")
    }

    async fn trigger_http(
        &mut self,
        _self_: wasmtime::component::Resource<component::spin_test_runner::spin::Instance>,
        _req: wasmtime::component::Resource<HttpRequest>,
    ) -> wasmtime::Result<wasmtime::component::Resource<HttpResponse>> {
        todo!("Instance::trigger_http was called")
    }

    fn drop(
        &mut self,
        _rep: wasmtime::component::Resource<component::spin_test_runner::spin::Instance>,
    ) -> wasmtime::Result<()> {
        todo!()
    }
}
impl component::spin_test_runner::spin::Host for View {}

impl wasmtime_wasi_http::WasiHttpView for View {
    fn ctx(&mut self) -> &mut wasmtime_wasi_http::WasiHttpCtx {
        &mut self.http_ctx
    }

    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        &mut self.table
    }
}
