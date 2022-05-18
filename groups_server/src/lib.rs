#[macro_use]
extern crate rocket;

use std::path::PathBuf;

use itertools::Itertools;
use rocket::fs::{relative, FileServer, NamedFile};
use rocket::{Build, Rocket};

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

#[get("/random")]
async fn random() -> String {
    let (students, _) = groups_core::random::random_students(50, None);
    students.iter().map(|s| s.encode()).join("\n")
}

async fn html_file_named(filename: &str) -> Option<NamedFile> {
    let mut path = PathBuf::from(relative!("static"));
    path.push("html");
    path.push(format!("{filename}.html"));
    NamedFile::open(path).await.ok()
}

pub async fn build_rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![index, student, instructor, random])
        .mount("/static", FileServer::from(relative!("static")))
}

#[shuttle_service::main]
async fn init() -> Result<Rocket<Build>, shuttle_service::Error> {
    let rocket = build_rocket().await;
    Ok(rocket)
}
