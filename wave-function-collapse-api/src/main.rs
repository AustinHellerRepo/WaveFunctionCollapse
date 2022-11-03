use tide::prelude::*;
use tide::http::mime::JSON;
use tide::Response;
use std::collections::{HashMap, HashSet};
use std::ops::Mul;
use std::sync::{Mutex, RwLock};
use async_std::sync::Arc;
type MultiThreadState = Arc<Mutex<ApiState>>;
mod wave_function;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[derive(Debug, Clone)]
struct ApiState {
    // NOP
}

#[derive(Debug, Deserialize)]
struct RequestCommand {
    nodes: Vec<wave_function::Node>,
    node_state_collections: Vec<wave_function::NodeStateCollection>
}

#[derive(Debug, Deserialize)]
struct ResponseCommand {
    request: RequestCommand,
    status: u32,
    content: String,
    headers: HashMap<String, String>
}
#[async_std::main]
async fn main() -> tide::Result<()> {

    pretty_env_logger::init();
    info!("started");

    let state = Arc::new(Mutex::new(ApiState {
    }));
    let mut app = tide::with_state(state);
    app.at("/").get(test_get);
    app.at("/").post(test_post);
    app.at("/collapse").post(post_request);
    app.listen("localhost:8080").await?;

    Ok(())
}

async fn test_get(req: tide::Request<MultiThreadState>) -> tide::Result {
    assert_eq!(req.header("X-TestHeader").expect("Header should contain custom header key"), "SomeGetValue");
    let mut resp = tide::Response::new(200);
    resp.set_body("Hello, world!");
    Ok(resp)
}

async fn test_post(req: tide::Request<MultiThreadState>) -> tide::Result {
    assert_eq!(req.header("X-TestHeader").expect("Header should contain custom header key"), "SomePostValue");
    let mut resp = tide::Response::new(200);
    resp.set_content_type(JSON);
    let body = tide::Body::from_json(&"{ \"just_a_post\": \"or at least it should be\" }").expect("The hard-coded string should map to a JSON object.");
    resp.set_body(body);
    Ok(resp)
}

async fn post_request(mut req: tide::Request<MultiThreadState>) -> tide::Result {
    dbg!(&req);
    //let RequestCommand { url, http_type, query, data, headers } = req.body_json().await?;
    let request_command: RequestCommand = req.body_json().await?;
    println!("received command and parsed to struct");
    dbg!(&request_command.nodes);

    let mut response: Response;

    let wave_function = wave_function::WaveFunction::new(request_command.nodes, request_command.node_state_collections);

    match wave_function.collapse() {
        Err(error_message) => {
            let mut response = Response::new(400);
            response.set_body(error_message);
            return Ok(response);
        }
        Ok(collapsed_wave_function) => {
            response = Response::new(200);
            let response_body: String = serde_json::to_string(&collapsed_wave_function).unwrap();
            response.set_body(response_body);
        }
    }

    return Ok(response);
}