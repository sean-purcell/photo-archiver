use std::path::PathBuf;

use diesel::{sqlite::SqliteConnection, Connection};
use diesel_migrations::embed_migrations;
use eyre::{Report, Result, WrapErr};

use diesel::dsl::*;
use diesel::prelude::*;

mod models;
mod schema;

pub use models::Media;

embed_migrations!();

pub struct Metadata {
    db: SqliteConnection,
}

impl Metadata {
    pub fn create(path: impl Into<PathBuf>) -> Result<Self> {
        let mut path = path.into();
        path.push("metadata.db");

        let path_str = path
            .to_str()
            .ok_or_else(|| Report::msg("Path includes non-unicode characters"))?;
        let db =
            SqliteConnection::establish(path_str).wrap_err("Failed to open metadata database")?;

        embedded_migrations::run(&db)?;

        Ok(Metadata { db })
    }

    pub fn insert(&self, item: &Media) -> Result<()> {
        use schema::media::dsl::*;
        insert_into(media).values(item).execute(&self.db)?;
        Ok(())
    }

    pub fn find(&self, key: &str) -> Result<Option<Media>> {
        use schema::media::dsl::*;
        Ok(media.find(key).first(&self.db).optional()?)
    }

    pub fn exists(&self, id: &str) -> Result<bool> {
        Ok(self.find(id)?.is_some())
    }
}
