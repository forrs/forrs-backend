#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{get, routes, State};
use std::fs;

mod config;
use config::*;
mod error;
use error::Error;

mod rest;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[get("/config")]
async fn config(conf: State<'_, Config>) -> String {
    format!("{:#?}", conf)
}

fn main() -> Result<(), Error> {
    let config: Config = fs::read_to_string("Forrs.toml")?.parse()?;
    Ok(rocket::ignite()
        .manage(config)
        .mount(
            "/",
            routes![
                index,
                config,
                rest::all_categories,
                rest::category_by_id,
                rest::category_by_name,
                rest::new_category
            ],
        )
        .launch()?)
}
