#[allow(dead_code)]
mod bindings;

use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
};

use bindings::exports::component::spin_test_runner::{
    host_impls::GuestKeyValue,
    runner::{self, GuestRuntime, GuestRuntimeBuilder, HttpRequest, HttpResponse},
};
use bindings::{
    component::spin_test_runner::spin::Instance as InnerInstance,
    exports::component::spin_test_runner::host_impls,
};

struct Guest;

impl host_impls::Guest for Guest {
    type KeyValue = KeyValue;
}
impl runner::Guest for Guest {
    type Runtime = Runtime;
    type RuntimeBuilder = RuntimeBuilder;
}

pub struct RuntimeBuilder {
    manifest_path: PathBuf,
    kv: RefCell<Option<host_impls::KeyValue>>,
}

impl GuestRuntimeBuilder for RuntimeBuilder {
    fn new() -> Self {
        let manifest_path = find_manifest_path()
            .expect("error finding manifest path")
            .expect("no manifest found in any parent directory");
        Self {
            manifest_path,
            kv: RefCell::new(None),
        }
    }

    fn key_value(&self, key_value: host_impls::KeyValue) {
        *self.kv.borrow_mut() = Some(key_value);
    }

    fn build(&self) -> runner::Runtime {
        let locked_app = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                spin_loader::from_file(
                    &self.manifest_path,
                    spin_loader::FilesMountStrategy::Direct,
                    Some("/".into()),
                )
                .await
            })
            .unwrap();
        let json = locked_app.to_json().unwrap();
        println!("JSON: {}", json.len());
        // std::fs::write(env.path().join("locked.json"), json)?;
        // let loader = TriggerLoader::new(env.path().join(".working_dir"), false);
        // let mut builder = TriggerExecutorBuilder::<HttpTrigger>::new(loader);
        // TODO(rylev): see if we can reuse the builder from spin_trigger instead of duplicating it here
        // builder.hooks(spin_trigger::network::Network);
        // let trigger = builder
        //     .build(
        //         format!("file:{}", env.path().join("locked.json").display()),
        //         RuntimeConfig::default(),
        //         HostComponentInitData::default(),
        //     )
        //     .await?;

        // The trigger has a `handle` method on it that can be used to invoke the Spin app
        // We would use that from the `trigger_http` method below.
        let spin = Instance::up(&self.manifest_path.display().to_string()).unwrap();
        let runtime = Runtime { instance: spin };
        runner::Runtime::new(runtime)
    }
}

pub struct Runtime {
    instance: Instance,
}

impl GuestRuntime for Runtime {
    fn trigger_http(&self, req: HttpRequest) -> HttpResponse {
        self.instance.trigger_http(req)
    }
}

pub struct KeyValue {
    values: RefCell<HashMap<String, Vec<u8>>>,
}

impl GuestKeyValue for KeyValue {
    fn new() -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
        }
    }

    fn set(&self, key: String, value: Vec<u8>) {
        self.values.borrow_mut().insert(key, value);
    }

    fn state(&self) -> Vec<(String, Vec<u8>)> {
        self.values
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

fn find_manifest_path() -> std::io::Result<Option<PathBuf>> {
    fn find_file_in(path: &Path, file: &str) -> Option<PathBuf> {
        let file_path = path.join(file);
        if file_path.exists() {
            Some(file_path.to_owned())
        } else {
            let parent = path.parent()?;
            find_file_in(parent, file)
        }
    }
    Ok(find_file_in(&std::env::current_dir()?, "spin.toml"))
}

struct Instance {
    inner: InnerInstance,
}

impl Instance {
    fn up(_manifest_path: &str) -> Result<Self, String> {
        // TODO: go from manifest_path to running Spin test instance
        Ok(Self {
            inner: InnerInstance::create()?,
        })
    }

    fn trigger_http(&self, req: HttpRequest) -> HttpResponse {
        self.inner.trigger_http(req)
    }
}

bindings::export!(Guest with_types_in bindings);
