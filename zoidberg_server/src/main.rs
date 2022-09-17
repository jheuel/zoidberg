use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use clap;
use std::sync::Mutex;

use zoidberg_lib::types::{Job, RegisterResponse, StatusRequest, Update};

struct State {
    counter: Mutex<i32>,
    jobcounter: Mutex<i32>,
    workers: Mutex<Vec<i32>>,
    jobs: Mutex<Vec<Job>>,
    running_jobs: Mutex<Vec<Job>>,
}

#[get("/")]
async fn index(data: web::Data<State>) -> impl Responder {
    let workers = data.workers.lock().unwrap();
    let jobs = data.jobs.lock().unwrap();

    let s: String = workers
        .iter()
        .map(|w| w.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    let s: String = s + &jobs
        .iter()
        .map(|j| serde_json::to_string(&j).unwrap())
        .collect::<Vec<String>>()
        .join("\n");
    HttpResponse::Ok().body(s)
}

#[get("/register")]
async fn register(data: web::Data<State>) -> Result<impl Responder> {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    let mut workers = data.workers.lock().unwrap();
    workers.push(*counter);

    println!("Registered worker node with id: {}", *counter);
    Ok(web::Json(RegisterResponse { id: *counter }))
}

#[get("/fetch")]
async fn fetch(data: web::Data<State>) -> Result<impl Responder> {
    let mut jobs = data.jobs.lock().unwrap();
    if let Some(j) = jobs.pop() {
        return Ok(web::Json(vec![j]));
    }
    Ok(web::Json(Vec::new()))
}

#[post("/status")]
async fn status(
    s: web::Json<Vec<StatusRequest>>,
    data: web::Data<State>,
) -> Result<impl Responder> {
    let running_jobs = data.running_jobs.lock().unwrap();
    let status_updates: Vec<Job> = running_jobs
        .iter()
        .filter(|r| {
            s.iter().filter(|i| i.id == r.id).count() > 0
        })
        .cloned()
        .collect();

    Ok(web::Json(status_updates))
}

#[post("/update")]
async fn update(updates: web::Json<Vec<Update>>, data: web::Data<State>) -> Result<String> {
    let mut running_jobs = data.running_jobs.lock().unwrap();
    let mut n = 0;
    for update in updates.iter() {
        println!(
            "Worker {} updated job {} with status {}",
            update.worker, update.job, update.status
        );
        for i in 0..running_jobs.len() {
            if running_jobs[i].id == update.job {
                running_jobs[i].status = update.status.clone();
            }
        }
        n += 1;
    }
    Ok(format!("Worker updated {} job(s)", n))
}

#[post("/submit")]
async fn submit(data: web::Data<State>, js: web::Json<Vec<Job>>) -> Result<impl Responder> {
    let mut jobs = data.jobs.lock().unwrap();
    let mut running_jobs = data.running_jobs.lock().unwrap();
    let mut jobcounter = data.jobcounter.lock().unwrap();
    let mut new_jobs = Vec::new();
    for j in js.into_inner() {
        *jobcounter += 1;
        let cmd = j.cmd.clone();
        println!("Job submitted with id: {}, cmd: {}", *jobcounter, cmd);

        new_jobs.push(Job {
            id: *jobcounter,
            ..j
        });
    }
    for job in new_jobs.iter() {
        jobs.push(job.clone());
        running_jobs.push(job.clone());
    }
    Ok(web::Json(new_jobs))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let _matches = clap::App::new("Zoidberg server")
        .version(VERSION)
        .author("Johannes Heuel")
        .get_matches();

    let state = web::Data::new(State {
        counter: Mutex::new(0),
        jobcounter: Mutex::new(0),
        workers: Mutex::new(Vec::new()),
        jobs: Mutex::new(Vec::new()),
        running_jobs: Mutex::new(Vec::new()),
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
                    counter: Mutex::new(0),
                    jobcounter: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    jobs: Mutex::new(Vec::new()),
                    running_jobs: Mutex::new(Vec::new()),
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
                    counter: Mutex::new(0),
                    jobcounter: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    jobs: Mutex::new(Vec::new()),
                    running_jobs: Mutex::new(Vec::new()),
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
                    counter: Mutex::new(0),
                    jobcounter: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    jobs: Mutex::new(vec![Job {
                        id: 0,
                        cmd: cmd.clone(),
                        status: Status::Submitted,
                    }]),
                    running_jobs: Mutex::new(Vec::new()),
                }))
                .service(fetch),
        )
        .await;
        let req = test::TestRequest::get().uri("/fetch").to_request();
        let resp: Vec<Job> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp[0].id, 0);
        assert_eq!(resp[0].cmd, cmd);

        let req = test::TestRequest::get().uri("/fetch").to_request();
        let resp: Vec<Job> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 0);
    }

    #[actix_web::test]
    async fn test_status() {
        let cmd = String::from("hi");
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(State {
                    counter: Mutex::new(0),
                    jobcounter: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    jobs: Mutex::new(vec![Job {
                        id: 0,
                        cmd: cmd.clone(),
                        status: Status::Running,
                    }]),
                    running_jobs: Mutex::new(Vec::new()),
                }))
                .service(status),
        )
        .await;
        let req = test::TestRequest::post()
            .set_json(vec![StatusRequest { id: 0 }])
            .uri("/status")
            .to_request();
        let resp: Vec<Job> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp[0].id, 0);
    }

    #[actix_web::test]
    async fn test_update() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(State {
                    counter: Mutex::new(0),
                    jobcounter: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    jobs: Mutex::new(Vec::new()),
                    running_jobs: Mutex::new(Vec::new()),
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
                    counter: Mutex::new(0),
                    jobcounter: Mutex::new(0),
                    workers: Mutex::new(Vec::new()),
                    jobs: Mutex::new(Vec::new()),
                    running_jobs: Mutex::new(Vec::new()),
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
