use rocket::{
    http::{ContentType, Status},
    request::Request,
    response::{self, Responder, Response},
};
use snafu::Snafu;
use std::io::Cursor;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error {}: {}", context, source))]
    General {
        context: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[snafu(display("Database error {}: {}", context, source))]
    DbError {
        context: String,
        source: tokio_postgres::Error,
    },
    #[snafu(display("A {} must have a unique {}!", table, field))]
    NonUniqueName { table: String, field: String },
    #[snafu(display("Not found: {}", context))]
    NotFound { context: String },
}

impl Error {
    fn http_status(&self) -> Status {
        match self {
            Error::NonUniqueName { .. } => Status::BadRequest,
            Error::NotFound { .. } => Status::NotFound,
            Error::General { .. } | Error::DbError { .. } => Status::InternalServerError,
        }
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _: &'r Request) -> response::ResultFuture<'r> {
        Box::pin(async move {
            Response::build()
                .status(self.http_status())
                .header(ContentType::Plain)
                .sized_body(Cursor::new(format!("{}", self)))
                .ok()
        })
    }
}
