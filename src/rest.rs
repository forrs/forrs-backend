use rocket::{get, post};
use rocket_contrib::json::Json;

use crate::{db, error::user::*};
use forrs_data::{id::Id, Category};

#[get("/category/<id>")]
pub async fn category_by_id(id: u64, client: &db::Client) -> Result<Json<Category>, Error> {
    client
        .fetch_item_opt("SELECT * FROM Category WHERE id = $1", &[&Id::from(id)])
        .await?
        .map(|category| Ok(Json(category)))
        .unwrap_or_else(|| {
            Err(Error::NotFound {
                context: "Category does not exist".into(),
            })
        })
}

#[get("/category/<name>", rank = 2)]
pub async fn category_by_name(name: String, client: &db::Client) -> Result<Json<Category>, Error> {
    client
        .fetch_item_opt("SELECT * FROM Category WHERE name = $1", &[&name])
        .await?
        .map(|category| Ok(Json(category)))
        .unwrap_or_else(|| {
            Err(Error::NotFound {
                context: "Category does not exist".into(),
            })
        })
}

#[get("/categories")]
pub async fn all_categories(client: &db::Client) -> Result<Json<Vec<Category>>, Error> {
    client.fetch_all_items().await.map(Json)
}

#[post("/category/<name>")]
pub async fn new_category(name: String, client: &db::Client) -> Result<Json<u64>, Error> {
    client.insert(&Category::new(name)).await.map(Json)
}
