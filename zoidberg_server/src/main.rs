use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use clap;
use env_logger::Env;
use log;
use std::sync::Mutex;
use zoidberg_lib::types::{FetchResponse, Job, RegisterResponse, StatusRequest, Update, Worker};

mod webpage;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct State {
    counter_workers: Mutex<i32>,
    counter_jobs: Mutex<i32>,
    workers: Mutex<Vec<Worker>>,
    new_jobs: Mutex<Vec<Job>>,
    jobs: Mutex<Vec<Job>>,
}

#[get("/")]
async fn index(data: web::Data<State>) -> impl Responder {
    let workers = data.workers.lock().unwrap();
    let new_jobs = data.new_jobs.lock().unwrap();
    let page = webpage::render(&*new_jobs, &*workers);
    HttpResponse::Ok().body(page)
}

#[get("/register")]
async fn register(data: web::Data<State>) -> Result<impl Responder> {
    let mut counter_workers = data.counter_workers.lock().unwrap();
    *counter_workers += 1;

    let mut workers = data.workers.lock().unwrap();
    workers.push(Worker {
        id: *counter_workers,
    });

    log::info!("Registered worker node with id: {}", *counter_workers);
    Ok(web::Json(RegisterResponse {
        id: *counter_workers,
    }))
}

#[get("/fetch")]
async fn fetch(data: web::Data<State>) -> Result<impl Responder> {
    let mut new_jobs = data.new_jobs.lock().unwrap();
    if let Some(j) = new_jobs.pop() {
        return Ok(web::Json(FetchResponse::Jobs(vec![j])));
    }
    Ok(web::Json(FetchResponse::Nop))
}

#[post("/status")]
async fn status(
    s: web::Json<Vec<StatusRequest>>,
    data: web::Data<State>,
) -> Result<impl Responder> {
    let jobs = data.jobs.lock().unwrap();
    let status_updates: Vec<Job> = jobs
        .iter()
        .filter(|r| s.iter().filter(|i| i.id == r.id).count() > 0)
        .cloned()
        .collect();

    Ok(web::Json(status_updates))
}

#[post("/update")]
async fn update(updates: web::Json<Vec<Update>>, data: web::Data<State>) -> Result<String> {
    let mut jobs = data.jobs.lock().unwrap();
    let mut n = 0;
    for update in updates.iter() {
        log::info!(
            "Worker {} updated job {} with status {}",
            update.worker,
            update.job,
            update.status
        );
        for i in 0..jobs.len() {
            if jobs[i].id == update.job {
                jobs[i].status = update.status.clone();
            }
        }
        n += 1;
    }
    Ok(format!("Worker updated {} job(s)", n))
}

#[post("/submit")]
async fn submit(data: web::Data<State>, js: web::Json<Vec<Job>>) -> Result<impl Responder> {
    let mut new_jobs = data.new_jobs.lock().unwrap();
    let mut jobs = data.jobs.lock().unwrap();
    let mut counter_jobs = data.counter_jobs.lock().unwrap();
    let mut new_new_jobs = Vec::new();
    for j in js.into_inner() {
        *counter_jobs += 1;
        let cmd = j.cmd.clone();
        log::info!("Job submitted with id: {}, cmd: {}", *counter_jobs, cmd);

        new_new_jobs.push(Job {
            id: *counter_jobs,
            ..j
        });
    }
    for job in new_new_jobs.iter() {
        new_jobs.push(job.clone());
        jobs.push(job.clone());
    }
    Ok(web::Json(new_new_jobs))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("zoidberg_server=info")).init();

    let _matches = clap::App::new("Zoidberg server")
        .version(VERSION)
        .author("Johannes Heuel")
        .get_matches();

    let state = web::Data::new(State {
        counter_workers: Mutex::new(0),
        counter_jobs: Mutex::new(0),
        workers: Mutex::new(Vec::new()),
        new_jobs: Mutex::new(Vec::new()),
        jobs: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(index)
            .service(register)
            .service(fetch)
            .service(status)
            .service(update)
            .service(submit)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, web, App};
    use zoidberg_lib::types::Status;

    #[actix_web::test]
    async fn test_index() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(State {
                    counter_workers: Mutex::new(0),
                    counter_jobs: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    new_jobs: Mutex::new(Vec::new()),
                    jobs: Mutex::new(Vec::new()),
                }))
                .service(index),
        )
        .await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_register() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(State {
                    counter_workers: Mutex::new(0),
                    counter_jobs: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    new_jobs: Mutex::new(Vec::new()),
                    jobs: Mutex::new(Vec::new()),
                }))
                .service(register),
        )
        .await;
        let req = test::TestRequest::get().uri("/register").to_request();
        let resp: RegisterResponse = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.id, 1);
    }

    #[actix_web::test]
    async fn test_fetch() {
        let cmd = String::from("hi");
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(State {
                    counter_workers: Mutex::new(0),
                    counter_jobs: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    new_jobs: Mutex::new(vec![Job {
                        id: 0,
                        cmd: cmd.clone(),
                        status: Status::Submitted,
                    }]),
                    jobs: Mutex::new(Vec::new()),
                }))
                .service(fetch),
        )
        .await;
        let req = test::TestRequest::get().uri("/fetch").to_request();
        let resp: FetchResponse = test::call_and_read_body_json(&app, req).await;
        match resp {
            FetchResponse::Nop => {
                panic!("did not expect FetchResponse::Nop")
            }
            FetchResponse::StopWorking => {
                panic!("did not expect FetchResponse::NotWorking")
            }
            FetchResponse::Jobs(new_jobs) => {
                assert_eq!(new_jobs[0].id, 0);
                assert_eq!(new_jobs[0].cmd, cmd);
            }
        }
    }

    #[actix_web::test]
    async fn test_status() {
        let cmd = String::from("hi");
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(State {
                    counter_workers: Mutex::new(0),
                    counter_jobs: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    new_jobs: Mutex::new(Vec::new()),
                    jobs: Mutex::new(vec![Job {
                        id: 1,
                        cmd: cmd.clone(),
                        status: Status::Running,
                    }]),
                }))
                .service(status),
        )
        .await;
        let req = test::TestRequest::post()
            .set_json(vec![StatusRequest { id: 1 }])
            .uri("/status")
            .to_request();
        let resp: Vec<Job> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp[0].id, 1);
    }

    #[actix_web::test]
    async fn test_update() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(State {
                    counter_workers: Mutex::new(0),
                    counter_jobs: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    new_jobs: Mutex::new(Vec::new()),
                    jobs: Mutex::new(Vec::new()),
                }))
                .service(update),
        )
        .await;
        let req = test::TestRequest::post()
            .set_json(vec![Update {
                worker: 0,
                job: 0,
                status: Status::Running,
            }])
            .uri("/update")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_submit() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(State {
                    counter_workers: Mutex::new(0),
                    counter_jobs: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    new_jobs: Mutex::new(Vec::new()),
                    jobs: Mutex::new(Vec::new()),
                }))
                .service(submit),
        )
        .await;
        let req = test::TestRequest::post()
            .set_json(vec![Job {
                id: 0,
                cmd: String::from("hi"),
                status: Status::Running,
            }])
            .uri("/submit")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }
}
