use clap::{App, Arg};
use env_logger::Env;
use futures::future::{AbortHandle, Abortable};
use log;
use reqwest::{header, Client, ClientBuilder};
use std::error::Error;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::{process::Command, time};

use zoidberg_lib::types::{
    FetchRequest, FetchResponse, Heartbeat, Job, RegisterResponse, Status, Update,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn build_client(secret: &str) -> Client {
    let cookie = secret.to_string();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "cookie",
        header::HeaderValue::from_str(&cookie)
            .unwrap_or_else(|_| panic!("invalid header value {}", &cookie)),
    );

    ClientBuilder::new()
        .timeout(Duration::from_secs(15))
        .default_headers(headers)
        .build()
        .expect("Could not create client")
}

#[derive(Debug, Clone)]
struct Worker {
    id: String,
    secret: String,
    server: String,
}

impl Worker {
    async fn new(server: &str, secret: &str) -> Result<Worker, Box<dyn Error>> {
        let res = build_client(secret)
            .get(format!("{}/register", server))
            .send()
            .await?;

        let body = res.text().await?;
        let r: RegisterResponse = serde_json::from_str(&body)?;
        log::info!("registered worker with id: {}", &r.id);
        Ok(Worker {
            id: r.id,
            secret: secret.to_string(),
            server: server.to_string(),
        })
    }

    async fn update(&self, jobs: &[Job]) -> Result<(), Box<dyn Error>> {
        let updates: Vec<Update> = jobs
            .iter()
            .map(|job| Update {
                worker: self.id.clone(),
                job: job.id,
                status: job.status.clone(),
            })
            .collect();

        let body = build_client(&self.secret)
            .post(format!("{}/update", self.server))
            .json(&updates)
            .send()
            .await?
            .text()
            .await?;

        log::info!("Body: {}", body);
        Ok(())
    }

    async fn fetch(&self) -> Result<FetchResponse, Box<dyn Error>> {
        let res = build_client(&self.secret)
            .post(format!("{}/fetch", self.server))
            .json(&FetchRequest {
                worker_id: self.id.clone(),
            })
            .send()
            .await?;
        let body = res.text().await?;
        let resp: FetchResponse = serde_json::from_str(&body)?;
        Ok(resp)
    }

    async fn heartbeat(&self) {
        let _ = build_client(&self.secret)
            .post(format!("{}/heartbeat", self.server))
            .json(&Heartbeat {
                id: self.id.clone(),
            })
            .send()
            .await;
    }
}

async fn run(job: &Job) -> Result<(), Box<dyn Error>> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(&job.cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    let output = output.await?;

    log::info!(
        "command: {}\nstdout: {}\nstderr: {}",
        &job.cmd,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    match output.status.success() {
        true => Ok(()),
        false => Err(Box::from("Job failed")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let matches = App::new("Zoidberg client")
        .version(VERSION)
        .author("by Johannes Heuel")
        .arg(
            Arg::with_name("server")
                .takes_value(true)
                .required(true)
                .help("Set Zoidberg server address"),
        )
        .get_matches();
    let server = matches.value_of("server").unwrap();
    let secret = std::env::var("ZOIDBERG_SECRET").unwrap_or_else(|_| {
        eprintln!("Please set the $ZOIDBERG_SECRET environment variable");
        std::process::exit(1);
    });

    let client = Arc::new(
        Worker::new(server, &secret)
            .await
            .expect("Could not create client"),
    );

    let pause = time::Duration::from_secs(1);
    let long_pause = time::Duration::from_secs(40);
    let heartbeat_pause = time::Duration::from_secs(30);

    let (heartbeat_handle, abort_registration) = AbortHandle::new_pair();
    let c = Arc::clone(&client);
    tokio::spawn(Abortable::new(
        async move {
            loop {
                time::sleep(heartbeat_pause).await;
                c.heartbeat().await;
            }
        },
        abort_registration,
    ));

    let mut fail_counter = 0;
    loop {
        let jobs = if let Ok(fetch) = client.fetch().await {
            fail_counter = 0;
            match fetch {
                FetchResponse::Nop => {
                    time::sleep(pause).await;
                    continue;
                }
                FetchResponse::Terminate(m) => {
                    println!("Terminate worker: {}", m);
                    break;
                }
                FetchResponse::Jobs(jobs) => jobs,
            }
        } else {
            fail_counter += 1;
            if fail_counter == 3 {
                log::error!("failed to fetch three times, assume that server crashed and exit");
                std::process::exit(1);
            }
            log::error!("failed to fetch new jobs");
            time::sleep(long_pause).await;
            continue;
        };

        for job in jobs {
            let status = match run(&job).await {
                Ok(()) => Status::Completed,
                Err(..) => Status::Failed,
            };
            let update = &[Job {
                status,
                ..job.clone()
            }];
            if let Err(error) = client.update(update).await {
                log::info!("Could not update job: {}", error);
            }
        }
    }
    heartbeat_handle.abort();
    Ok(())
}
