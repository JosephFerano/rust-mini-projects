use actix_web::{get, web, App, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use std::process::Command;
use std::str;

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    age: u8,
    is_cool: bool,
}

#[get("/cowsay/{custom}")]
async fn cowsay_custom(path: web::Path<String>) -> impl Responder {
    Command::new("cowsay")
        .arg(path.into_inner())
        .output()
        .map_or(String::from("Could not find cowsay command"), |o| String::from_utf8(o.stdout).unwrap())
}

#[get("/cowsay/")]
async fn cowsay() -> impl Responder {
    let fortune = Command::new("fortune")
        .output()
        .map_or(String::from("Could not find fortune command"), |o| String::from_utf8(o.stdout).unwrap());
    Command::new("cowsay")
        .arg(fortune)
        .output()
        .map_or(String::from("Could not find cowsay command"), |o| String::from_utf8(o.stdout).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(||
        App::new()
            .service(cowsay)
            .service(cowsay_custom))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
