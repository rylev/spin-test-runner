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
    let component = std::env::args()
        .nth(1)
        .expect("Binary must be invoked with a path to the test runner component binary");

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
    let mut store = wasmtime::Store::new(&engine, view);
    let component = wasmtime::component::Component::from_file(&engine, component).unwrap();
    let mut linker = wasmtime::component::Linker::new(&engine);
    wasmtime_wasi::preview2::command::add_to_linker(&mut linker).unwrap();
    component::spin_test_runner::spin::add_to_linker(&mut linker, |x| x).unwrap();
    wasmtime_wasi_http::bindings::http::types::add_to_linker(&mut linker, |x| x).unwrap();
    let instance = linker
        .instantiate_async(&mut store, &component)
        .await
        .unwrap();
    let tr = TestRunner::new(&mut store, &instance).unwrap();
    let g = tr.component_spin_test_runner_runner();
    let runtime_builder = g
        .runtime_builder()
        .call_constructor(&mut store)
        .await
        .unwrap();
    let runtime_builder = runtime_builder
        .try_into_resource::<RuntimeBuilder>(&mut store)
        .unwrap();
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

use component::spin_test_runner::spin::Instance;
use component::spin_test_runner::spin::{HttpRequest, HttpResponse};
use exports::component::spin_test_runner::runner::RuntimeBuilder;

#[async_trait::async_trait]
impl component::spin_test_runner::spin::HostInstance for View {
    async fn create(
        &mut self,
    ) -> wasmtime::Result<Result<wasmtime::component::Resource<Instance>, String>> {
        todo!()
    }

    async fn trigger_http(
        &mut self,
        self_: wasmtime::component::Resource<Instance>,
        req: wasmtime::component::Resource<HttpRequest>,
    ) -> wasmtime::Result<wasmtime::component::Resource<HttpResponse>> {
        todo!()
    }

    fn drop(&mut self, rep: wasmtime::component::Resource<Instance>) -> wasmtime::Result<()> {
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
