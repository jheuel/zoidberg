use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use std::sync::Mutex;

use zoidberg_lib::types::Update;

struct State {
    counter: Mutex<i32>,
}

#[get("/register")]
async fn register(data: web::Data<State>) -> impl Responder {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    HttpResponse::Ok().body(format!("Worker node {} registered", *counter))
}

#[get("/fetch")]
async fn fetch() -> impl Responder {
    HttpResponse::Ok().body("Here is some work")
}

#[post("/update")]
async fn update(u: web::Json<Update>) -> Result<String> {
    Ok(format!("Job {} updated with status {}", u.id, u.status))
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(State {
        counter: Mutex::new(0),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .service(register)
            .service(fetch)
            .service(update)
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
