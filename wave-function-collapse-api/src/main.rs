use tide::prelude::*;
use tide::http::mime::JSON;
use tide::Response;
use std::collections::{HashMap, HashSet};
use std::ops::Mul;
use std::sync::{Mutex, RwLock};
use async_std::sync::Arc;
type MultiThreadState = Arc<Mutex<ApiState>>;
mod wave_function;

#[derive(Debug, Clone)]
struct ApiState {
    // NOP
}

#[derive(Debug, Deserialize)]
struct RequestCommand {
    states: Vec<wave_function::NodeState>,
    nodes: Vec<wave_function::Node>
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
    dbg!(&request_command.states);

    let wave_function = wave_function::WaveFunction::new(request_command.states, request_command.nodes);
    let error_message = wave_function.validate();

    if let Some(message) = error_message {
        let mut response = Response::new(400);
        response.set_body(message);
        return Ok(response);
    }

    let collapsed_wave_function = wave_function.collapse().expect("The wave function should collapse to a set of nodes with specific states.");

    // TODO convert the collapsed wave function to a JSON object

    let mut response = Response::new(500);
    response.set_body("not implemented");
    return Ok(response);
}