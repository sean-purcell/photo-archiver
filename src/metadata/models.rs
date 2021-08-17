use std::path::Path;

use chrono::{offset::Utc, DateTime, NaiveDateTime};

use crate::item::Item;

use super::schema::*;

#[derive(Clone, Debug, Insertable, Queryable)]
#[table_name = "media"]
pub struct Media {
    id: String,
    file_path: String,
    creation_date: NaiveDateTime,
    download_date: NaiveDateTime,
}

impl Media {
    pub fn new(
        id: &str,
        file_path: impl AsRef<Path>,
        creation_time: DateTime<Utc>,
        download_time: DateTime<Utc>,
    ) -> Self {
        Media {
            id: item.id().to_string(),
            file_path: file_path.to_string(),
            creation_date: creation_time.naive_utc(),
            download_date: download_time.naive_utc(),
        }
    }
}
