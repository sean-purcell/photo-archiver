use std::path::Path;

use eyre::{Report, Result};
use futures::stream::TryStreamExt;
use reqwest::Client;
use tokio::{fs::File, io::AsyncWriteExt};

pub async fn download(client: &Client, url: &str, file: impl AsRef<Path>) -> Result<()> {
    let file = File::create(file).await?;
    let response = client.get(url).send().await?;

    let _finished_file = response
        .bytes_stream()
        .map_err(Report::new)
        .try_fold(file, {
            |mut file, bytes| async move {
                file.write_all(&*bytes).await.map_err(Report::new)?;
                Ok(file)
            }
        })
        .await?;

    Ok(())
}
