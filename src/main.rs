#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{get, routes};
use std::fs;

mod config;
use config::*;
mod error;
use error::launch;

mod rest;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

fn main() -> Result<(), launch::Error> {
    let config: Config = fs::read_to_string("Forrs.toml")?.parse()?;
    Ok(rocket::ignite()
        .manage(config.database)
        .mount(
            "/",
            routes![
                index,
                rest::all_categories,
                rest::category_by_id,
                rest::category_by_name,
                rest::new_category
            ],
        )
        .launch()?)
}
