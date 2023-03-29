#[macro_use]
extern crate rocket;

use rocket::fs::{relative, FileServer, NamedFile};
use rocket::{Build, Rocket};
use std::path::PathBuf;

#[get("/")]
async fn index() -> Option<NamedFile> {
    html_file_named("index").await
}

#[get("/student")]
async fn student() -> Option<NamedFile> {
    html_file_named("student").await
}

#[get("/instructor")]
async fn instructor() -> Option<NamedFile> {
    html_file_named("instructor").await
}

async fn html_file_named(filename: &str) -> Option<NamedFile> {
    let mut path = PathBuf::from(relative!("static"));
    path.push("html");
    path.push(format!("{filename}.html"));
    NamedFile::open(path).await.ok()
}

pub async fn build_rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![index, student, instructor])
        .mount("/static", FileServer::from(relative!("static")))
}

#[shuttle_runtime::main]
async fn init() -> shuttle_rocket::ShuttleRocket {
    let rocket = build_rocket().await;
    Ok(rocket.into())
}
