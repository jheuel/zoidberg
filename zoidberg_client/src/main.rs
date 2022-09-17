use clap::{App, Arg};
use reqwest::{header, Client, ClientBuilder};
use std::error::Error;
use std::process::Command;
use std::time::Duration;
use std::{thread, time};

use zoidberg_lib::types::{Job, RegisterResponse, Status, Update};

fn build_client(secret: &str) -> Client {
    let cookie = format!("secret={}", secret);

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

#[derive(Debug)]
struct Worker {
    id: i32,
}

impl Worker {
    async fn new(server: &str) -> Result<Worker, Box<dyn Error>> {
        let res = build_client("some_secret")
            .get(format!("{}/register", server))
            .send()
            .await?;

        let body = res.text().await?;
        let r: RegisterResponse = serde_json::from_str(&body)?;
        println!("registered worker with id: {}", &r.id);
        Ok(Worker { id: r.id })
    }

    async fn update(self: &Self, jobs: &[Job]) -> Result<(), Box<dyn Error>> {
        let updates: Vec<Update> = jobs
            .iter()
            .map(|job| Update {
                worker: self.id,
                job: job.id,
                status: job.status.clone(),
            })
            .collect();

        let body = build_client("some_secret")
            .post("http://localhost:8080/update")
            .json(&updates)
            .send()
            .await?
            .text()
            .await?;

        println!("Body:\n{}", body);
        Ok(())
    }

    async fn fetch(self: &Self) -> Result<Vec<Job>, Box<dyn Error>> {
        let res = build_client("some_secret")
            .get("http://localhost:8080/fetch")
            .send()
            .await?;
        let body = res.text().await?;
        let jobs: Vec<Job> = serde_json::from_str(&body)?;
        Ok(jobs)
    }

    async fn run(self: &Self, job: &Job) -> Result<(), Box<dyn Error>> {
        let output = Command::new("bash").arg("-c").arg(&job.cmd).output()?;

        println!(
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let matches = App::new("Zoidberg client")
        .version(VERSION)
        .author("Johannes Heuel")
        .arg(
            Arg::with_name("server")
                .takes_value(true)
                .required(true)
                .help("Set Zoidberg server address"),
        )
        .get_matches();
    let server = matches.value_of("server").unwrap();

    let client = Worker::new(server).await.expect("Could not create client");

    let pause = time::Duration::from_secs(0);
    let long_pause = time::Duration::from_secs(2);
    let extra_long_pause = time::Duration::from_secs(4);

    loop {
        if let Ok(jobs) = client.fetch().await {
            // if there are no jobs, wait a little longer
            if jobs.len() == 0 {
                thread::sleep(long_pause);
            }

            for job in jobs {
                let status = match client.run(&job).await {
                    Ok(()) => Status::Completed,
                    Err(..) => Status::Failed,
                };
                let n = &[Job { status, ..job }];
                if let Err(error) = client.update(n).await {
                    println!("Could not update job: {}", error);
                }
            }
        } else {
            // wait a little longer whenever job fetching fails
            thread::sleep(extra_long_pause);
        }
        thread::sleep(pause);
    }
}
