#[macro_use] extern crate rocket;

#[launch]
async fn rocket() -> _ {
    groups_server::build_rocket().await
}