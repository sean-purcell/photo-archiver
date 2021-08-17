use chrono::NaiveDateTime;

use super::schema::*;

#[derive(Clone, Debug, Insertable, Queryable)]
#[table_name = "media"]
pub struct Media {
    id: String,
    file_path: String,
    creation_date: NaiveDateTime,
    download_date: NaiveDateTime,
}
