use itertools::Itertools;

#[macro_use]
extern crate rocket;

#[get("/random")]
async fn random() -> String {
    // This lives only in the local server since shuttle doesn't want to build when using a path dependency.
    let (students, _) = groups_core::random::random_students(50, None);
    students.iter().map(|s| s.encode()).join("\n")
}

#[launch]
async fn rocket() -> _ {
    groups_server::build_rocket()
        .await
        .mount("/", routes![random])
}
