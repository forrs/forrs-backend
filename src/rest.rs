use rocket::{get, post};
use rocket::{http::Status, State};
use rocket_contrib::json::Json;
use tokio_postgres::Row;

use crate::config::Config;
use forrs_data::{
    id::Id,
    sql::{into_insert::IntoInsert, FromRow},
    Category,
};

mod helper {
    use crate::config::DbConfig;
    use futures::FutureExt;
    use rocket::http::Status;
    use tokio_postgres::{Client, Config as PGConfig, NoTls};
    pub async fn connect(conf: &DbConfig) -> Result<Client, Status> {
        let (client, connection) = PGConfig::from(conf)
            .connect(NoTls)
            .await
            .map_err(|_| Status::InternalServerError)?;
        let connection = connection.map(|r| {
            if let Err(e) = r {
                eprintln!("connection error: {}", e);
            }
        });
        tokio::spawn(connection);
        Ok(client)
    }
}

#[get("/category/<id>")]
pub async fn category_by_id(id: u64, conf: State<'_, Config>) -> Result<String, Status> {
    let client = helper::connect(&conf.database).await?;
    let stmt = client
        .prepare("SELECT * FROM Category WHERE id = $1")
        .await
        .map_err(|_| Status::InternalServerError)?;
    let rows: Vec<Row> = client
        .query(&stmt, &[&Id::from(id)])
        .await
        .map_err(|_| Status::InternalServerError)?;
    if let Some(row) = rows.first() {
        let category = Category::from_row(row).map_err(|_| Status::InternalServerError)?;
        Ok(serde_json::to_string_pretty(&category).unwrap())
    } else {
        Err(Status::NotFound)
    }
}

#[get("/category/<name>", rank = 2)]
pub async fn category_by_name(name: String, conf: State<'_, Config>) -> Result<String, Status> {
    let client = helper::connect(&conf.database).await?;
    let stmt = client
        .prepare("SELECT * FROM Category WHERE name = $1")
        .await
        .map_err(|_| Status::InternalServerError)?;
    let rows: Vec<Row> = client
        .query(&stmt, &[&name])
        .await
        .map_err(|_| Status::InternalServerError)?;
    if let Some(row) = rows.first() {
        let category = Category::from_row(row).map_err(|_| Status::InternalServerError)?;
        Ok(serde_json::to_string_pretty(&category).unwrap())
    } else {
        Err(Status::NotFound)
    }
}

#[get("/categories")]
pub async fn all_categories(conf: State<'_, Config>) -> Result<Json<Vec<Category>>, Status> {
    let client = helper::connect(&conf.database).await?;
    let rows: Vec<Row> = client
        .query("SELECT * FROM Category", &[])
        .await
        .map_err(|_| Status::InternalServerError)?;
    let mut result = Vec::new();
    for row in rows.iter() {
        result.push(Category::from_row(row).map_err(|_| Status::InternalServerError)?);
    }
    Ok(Json(result))
}

#[post("/category/<name>")]
pub async fn new_category(name: String, conf: State<'_, Config>) -> Result<String, Status> {
    let client = helper::connect(&conf.database).await?;
    let category = Category::new(name);
    let stmt = client
        .prepare(Category::insert_stmt())
        .await
        .map_err(|_| Status::InternalServerError)?;
    match client.execute(&stmt, &category.insert_params()).await {
        Ok(id) => Ok(format!("Id: {}", id)),
        Err(e) => {
            if let Some(sql_state) = e.code() {
                use tokio_postgres::error::SqlState;
                if sql_state == &SqlState::UNIQUE_VIOLATION {
                    return Err(Status::ImATeapot);
                }
            }
            Err(Status::InternalServerError)
        }
    }
}
