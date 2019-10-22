use rocket::State;
use rocket::{get, post};
use rocket_contrib::json::Json;

use crate::{config::Config, db, error::user::*};
use forrs_data::{id::Id, Category};

#[get("/category/<id>")]
pub async fn category_by_id(id: u64, conf: State<'_, Config>) -> Result<Json<Category>, Error> {
    let client = db::Client::connect(&conf.database).await?;
    let result = client
        .fetch_item_opt("SELECT * FROM Category WHERE id = $1", &[&Id::from(id)])
        .await?;
    if let Some(category) = result {
        Ok(Json(category))
    } else {
        Err(Error::NotFound {
            context: "Category does not exist".into(),
        })
    }
}

#[get("/category/<name>", rank = 2)]
pub async fn category_by_name(
    name: String,
    conf: State<'_, Config>,
) -> Result<Json<Category>, Error> {
    let client = db::Client::connect(&conf.database).await?;
    let result = client
        .fetch_item_opt("SELECT * FROM Category WHERE name = $1", &[&name])
        .await?;
    if let Some(category) = result {
        Ok(Json(category))
    } else {
        Err(Error::NotFound {
            context: "Category does not exist".into(),
        })
    }
}

#[get("/categories")]
pub async fn all_categories(conf: State<'_, Config>) -> Result<Json<Vec<Category>>, Error> {
    let client = db::Client::connect(&conf.database).await?;
    let result = client.fetch_all_items().await?;
    Ok(Json(result))
}

#[post("/category/<name>")]
pub async fn new_category(name: String, conf: State<'_, Config>) -> Result<Json<u64>, Error> {
    let client = db::Client::connect(&conf.database).await?;
    let category = Category::new(name);
    client.insert(&category).await.map(Json)
}
