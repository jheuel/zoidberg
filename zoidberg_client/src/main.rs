use core::time::Duration;
use reqwest::blocking::Client;
use reqwest::header;

use zoidberg_lib::types::Update;

fn build_client(secret: &str) -> Client {
    let cookie = format!("secret={}", secret);

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "cookie",
        header::HeaderValue::from_str(&cookie)
            .unwrap_or_else(|_| panic!("invalid header value {}", &cookie)),
    );

    Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(15))
        .build()
        .expect("Could not create HTTP client")
}

fn main() {
    // test get request to index
    let res = build_client("some_secret")
        .get("http://localhost:8080/")
        .send()
        .expect("Could not send get request");

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());

    let body = res.text().unwrap();
    println!("Body:\n{}", body);

    (0..10).for_each(|_| {
        // test get request to /register
        let res = build_client("some_secret")
            .get("http://localhost:8080/register")
            .send()
            .expect("Could not send get request");

        println!("Status: {}", res.status());
        println!("Headers:\n{:#?}", res.headers());
        let body = res.text().unwrap();
        println!("Body:\n{}", body);
    });

    // test post request to /update
    let update = Update {
        id: 99,
        status: "hi".to_string(),
    };

    let res = build_client("some_secret")
        .post("http://localhost:8080/update")
        .json(&update)
        .send()
        .expect("Could not send get request");

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    let body = res.text().unwrap();
    println!("Body:\n{}", body);
}
