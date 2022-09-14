use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct Update {
    id: i64,
    status: String,
}

#[get("/register")]
async fn register() -> impl Responder {
    HttpResponse::Ok().body("Worker node registered")
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
    HttpServer::new(|| {
        App::new()
            .service(register)
            .service(fetch)
            .service(update)
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
