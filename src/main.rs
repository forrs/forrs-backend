#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{get, routes};

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

fn main() -> Result<(), rocket::error::Error> {
    rocket::ignite().mount("/", routes![index]).launch()
}
