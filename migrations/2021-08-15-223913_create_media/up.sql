CREATE TABLE media (
    id            TEXT PRIMARY KEY NOT NULL,
    file_path     TEXT NOT NULL,
    creation_date DATETIME NOT NULL,
    download_date DATETIME NOT NULL
)
