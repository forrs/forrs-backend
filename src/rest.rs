use rocket::State;
use rocket::{get, post};
use rocket_contrib::json::Json;
use snafu::ResultExt;
use tokio_postgres::Row;

use crate::{config::Config, error::user::*};
use forrs_data::{
    id::Id,
    sql::{into_insert::IntoInsert, FromRow},
    Category,
};

mod helper {
    use crate::{config::DbConfig, error::user::*};
    use futures::FutureExt;
    use snafu::ResultExt;
    use tokio_postgres::{Client, Config as PGConfig, NoTls};

    pub async fn connect(conf: &DbConfig) -> Result<Client, Error> {
        let (client, connection) = PGConfig::from(conf).connect(NoTls).await.context(DbError {
            context: "connecting to database",
        })?;
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
pub async fn category_by_id(id: u64, conf: State<'_, Config>) -> Result<Json<Category>, Error> {
    let client = helper::connect(&conf.database).await?;
    let rows: Vec<Row> = client
        .query("SELECT * FROM Category WHERE id = $1", &[&Id::from(id)])
        .await
        .context(DbError {
            context: "loading category",
        })?;
    if let Some(row) = rows.first() {
        let category = Category::from_row(row).context(DbError {
            context: "parsing category",
        })?;
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
    let client = helper::connect(&conf.database).await?;
    let rows: Vec<Row> = client
        .query("SELECT * FROM Category WHERE name = $1", &[&name])
        .await
        .context(DbError {
            context: "loading category",
        })?;
    if let Some(row) = rows.first() {
        let category = Category::from_row(row).context(DbError {
            context: "parsing category",
        })?;
        Ok(Json(category))
    } else {
        Err(Error::NotFound {
            context: "Category does not exist".into(),
        })
    }
}

#[get("/categories")]
pub async fn all_categories(conf: State<'_, Config>) -> Result<Json<Vec<Category>>, Error> {
    let client = helper::connect(&conf.database).await?;
    let result = client
        .query("SELECT * FROM Category", &[])
        .await
        .context(DbError {
            context: "loading categories",
        })?
        .iter()
        .map(|row| Category::from_row(row))
        .collect::<Result<Vec<_>, _>>()
        .context(DbError {
            context: "parsing categories",
        })?;
    Ok(Json(result))
}

#[post("/category/<name>")]
pub async fn new_category(name: String, conf: State<'_, Config>) -> Result<Json<u64>, Error> {
    let client = helper::connect(&conf.database).await?;
    let category = Category::new(name);
    match client
        .execute(Category::insert_stmt(), &category.insert_params())
        .await
    {
        Ok(id) => Ok(Json(id)),
        Err(e) => {
            if let Some(sql_state) = e.code() {
                use tokio_postgres::error::SqlState;
                if sql_state == &SqlState::UNIQUE_VIOLATION {
                    return Err(Error::NonUniqueName {
                        table: "Category".into(),
                        value: category.name,
                    });
                }
            }
            Err(Error::DbError {
                context: "saving category".into(),
                source: e,
            })
        }
    }
}
