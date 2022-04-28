#[macro_use]
extern crate rocket;

use std::path::PathBuf;

use rocket::fs::{relative, FileServer, NamedFile};
use rocket::{Build, Rocket};

#[get("/")]
async fn index() -> Option<NamedFile> {
    let mut path = PathBuf::from(relative!("static"));
    path.push("html");
    path.push("index.html");

    NamedFile::open(path).await.ok()
}

#[get("/scheduler")]
async fn scheduler() -> Option<NamedFile> {
    let mut path = PathBuf::from(relative!("static"));
    path.push("html");
    path.push("scheduler.html");

    NamedFile::open(path).await.ok()
}

#[shuttle_service::main]
async fn init() -> Result<Rocket<Build>, shuttle_service::Error> {
    let rocket = rocket::build()
        .mount("/", routes![index, scheduler])
        .mount("/static", FileServer::from(relative!("static")));

    Ok(rocket)
}
