use deno_core::{JsRuntime, PollEventLoopOptions, RuntimeOptions};
use gadget_sdk::event_listener::EventListener;
use gadget_sdk::{job, config::GadgetConfiguration, Error};
use gadget_sdk::event_listener::periodic::PeriodicEventListener;
use tokio::runtime::Runtime;
use std::rc::Rc;
use tokio::fs;
use std::convert::Infallible;
struct ReturnsZero;

#[async_trait::async_trait]
impl EventListener<u64, MyContext> for ReturnsZero {
    async fn new(_context: &MyContext) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(Self)
    }

    async fn next_event(&mut self) -> Option<u64> {
        Some(0)
    }

    async fn handle_event(&mut self, _event: u64) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct MyContext;


/// Executes a TypeScript function (compiled to JavaScript) and returns the result.
#[job(
    id = 0,
    params(function_name),
    result(_),
    event_listener(PeriodicEventListener::<6000, ReturnsZero, u64, MyContext>),
    verifier(evm = "HelloBlueprint")
)]
pub async fn execute_js_fn(
    context: MyContext,
    function_name: String,
    env: GadgetConfiguration<parking_lot::RawRwLock>,
) -> Result<String, Infallible> {
    // Read the JavaScript file
    let js_code = fs::read_to_string("index.js").await.unwrap();

    // Move JavaScript execution to a blocking task
    let result = tokio::task::spawn_blocking(move || {
        let mut runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
            ..Default::default()
        });

        // Execute the JavaScript code to define the function
        runtime.execute_script("index.js", js_code).unwrap();

        // Create and execute the async function call
        let promise = runtime.execute_script(
            "call_do_something.js",
            "doSomething().then(result => { globalThis.asyncResult = result; })"
        ).unwrap();

        // Create a new tokio runtime to run the event loop
        let rt = Runtime::new().unwrap();
        rt.block_on(runtime.run_event_loop(PollEventLoopOptions::default())).unwrap();

        // Retrieve the result from the global scope
        let result = runtime.execute_script(
            "get_result.js",
            "globalThis.asyncResult"
        ).unwrap();

        // Convert the result to a Rust String
        let scope = &mut runtime.handle_scope();
        let result = deno_core::v8::Local::new(scope, result);
        result.to_rust_string_lossy(scope)
    }).await.unwrap();

    Ok(result)
}
