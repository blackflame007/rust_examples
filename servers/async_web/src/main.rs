use warp::{Filter, Rejection, Reply};
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

#[tokio::main]
async fn main() {
    // Define routes
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    let users = warp::path("users")
        .and(warp::get())
        .and_then(get_users);

    let delayed = warp::path!("delayed" / u64)
        .and_then(delayed_response);

    let routes = hello.or(users).or(delayed);

    println!("Server starting on http://localhost:3030");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn get_users() -> Result<impl Reply, Rejection> {
    let users = vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ];
    Ok(warp::reply::json(&users))
}

async fn delayed_response(seconds: u64) -> Result<impl Reply, Rejection> {
    sleep(Duration::from_secs(seconds)).await;
    Ok(format!("Response after {} second(s)", seconds))
}
