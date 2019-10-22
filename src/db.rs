use futures::FutureExt;
use snafu::ResultExt;
use tokio_postgres::{types::ToSql, Config as PGConfig, NoTls, ToStatement};

use crate::{config::DbConfig, error::user::*};
use forrs_data::sql::{FromRow, IntoInsert, Table};

pub struct Client {
    inner: tokio_postgres::Client,
}

impl Client {
    pub async fn connect(conf: &DbConfig) -> Result<Self, Error> {
        let (client, connection) = PGConfig::from(conf).connect(NoTls).await.context(DbError {
            context: "connecting to database",
        })?;
        let connection = connection.map(|r| {
            if let Err(e) = r {
                eprintln!("connection error: {}", e);
            }
        });
        tokio::spawn(connection);
        Ok(Self { inner: client })
    }

    pub async fn fetch_item_opt<T, S>(
        &self,
        stmt: &S,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<T>, Error>
    where
        T: FromRow,
        S: ToStatement + ?Sized,
    {
        let rows = self.inner.query(stmt, params).await.context(DbError {
            context: "loading item",
        })?;
        rows.first()
            .map(|row| {
                T::from_row(&row).context(DbError {
                    context: "parsing item",
                })
            })
            .transpose()
    }

    pub async fn fetch_items<T, S>(
        &self,
        stmt: &S,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<T>, Error>
    where
        T: FromRow,
        S: ToStatement + ?Sized,
    {
        self.inner
            .query(stmt, params)
            .await
            .context(DbError {
                context: "loading items",
            })?
            .iter()
            .map(|row| {
                T::from_row(row).context(DbError {
                    context: "parsing items",
                })
            })
            .collect()
    }

    pub async fn fetch_all_items<T>(&self) -> Result<Vec<T>, Error>
    where
        T: Table + FromRow,
    {
        let query = format!("SELECT * FROM {}", T::tablename());
        self.fetch_items(query.as_str(), &[]).await
    }

    pub async fn insert<T>(&self, item: &T) -> Result<u64, Error>
    where
        T: IntoInsert,
    {
        self.inner
            .execute(T::insert_stmt(), &item.insert_params())
            .await
            .or_else(|e| {
                if let Some(sql_state) = e.code() {
                    use tokio_postgres::error::SqlState;
                    if sql_state == &SqlState::UNIQUE_VIOLATION {
                        return Err(Error::NonUniqueName {
                            table: T::tablename().into(),
                            field: T::name_field().unwrap().into(),
                        });
                    }
                }
                Err(Error::DbError {
                    context: format!("saving {}", T::tablename()),
                    source: e,
                })
            })
    }
}
