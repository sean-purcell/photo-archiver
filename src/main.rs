#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::path::PathBuf;

use eyre::{Report, Result, WrapErr};
use futures::stream::{StreamExt, TryStreamExt};
use google_photoslibrary1::PhotosLibrary;
use log::LevelFilter;
use structopt::StructOpt;
use yup_oauth2::{
    authenticator::DefaultAuthenticator, noninteractive::NoninteractiveTokens,
    NoninteractiveAuthenticator,
};

mod archiver;
mod downloader;
mod item;
mod media_item_iter;
mod metadata;

use archiver::Archiver;
use item::Item;

#[derive(Debug, StructOpt)]
struct Auth {
    #[structopt(short = "t", long = "token")]
    token: String,
}

impl Auth {
    async fn authenticator(&self) -> Result<DefaultAuthenticator> {
        let contents =
            std::fs::read(&self.token).wrap_err(format!("Couldn't load file: {}", self.token))?;
        let token = serde_json::from_slice::<NoninteractiveTokens>(&contents)
            .wrap_err(format!("Failed to parse token file: {}", self.token))?;
        Ok(NoninteractiveAuthenticator::builder(token).build().await?)
    }
}

#[derive(enum_utils::FromStr, Debug, Clone, Copy)]
#[enumeration(case_insensitive)]
enum Style {
    Debug,
    FsPath,
    Json,
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to parse {0} as style")]
struct StyleParseError(String);

impl Style {
    fn serialize(self, item: &Item) -> String {
        use Style::*;
        match self {
            Debug => format!("{:?}", item.0),
            FsPath => item.fs_path().into_os_string().into_string().unwrap(),
            Json => serde_json::to_string(item).unwrap(),
        }
    }

    fn parse(s: &str) -> Result<Self, StyleParseError> {
        use std::str::FromStr;

        Self::from_str(s).map_err(|()| StyleParseError(s.into()))
    }
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, StructOpt)]
struct List {
    #[structopt(
        short = "n",
        long = "num",
        help = "Number of items to list",
        default_value = "50"
    )]
    num: usize,
    #[structopt(
        short = "s",
        long = "style",
        help = "Print style for item (options: debug, fspath, json)",
        default_value = "debug",
        parse(try_from_str = Style::parse),
    )]
    style: Style,
    #[structopt(
        short = "I",
        long = "no-index",
        help = "Don't print the index of each entry"
    )]
    no_index: bool,
}

impl List {
    async fn run(&self, hub: PhotosLibrary) -> Result<()> {
        let iter = media_item_iter::list(hub);
        iter.take(self.num)
            .enumerate()
            .map(|(i, val)| val.map(|item| (i, item)))
            .try_for_each(|(i, item)| async move {
                let rep = self.style.serialize(&(item.into()));
                let idx = if self.no_index {
                    "".to_string()
                } else {
                    format!("{}: ", i)
                };
                println!("{}{}", idx, rep);
                Ok(())
            })
            .await
            .wrap_err("Failed to list items")?;
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct Archive {
    #[structopt(
        short = "R",
        long = "root-directory",
        help = "Root directory to download photos to"
    )]
    root_dir: PathBuf,
    #[structopt(
        short = "c",
        long = "concurrency",
        help = "Max concurrent downloads",
        default_value = "4"
    )]
    concurrent_downloads: usize,
    #[structopt(
        short = "d",
        long = "dry-run",
        help = "Don't actually download any files or modify metadata"
    )]
    dry_run: bool,
}

impl Archive {
    async fn run(&self, hub: PhotosLibrary) -> Result<()> {
        let archiver = &Archiver::create(&self.root_dir)?;
        let items_iter = media_item_iter::list(&hub);

        let result = items_iter
            .map_err(Report::new)
            .try_for_each_concurrent(Some(self.concurrent_downloads), {
                |media_item| async move {
                    let archiver = archiver.clone();

                    let item = Item(media_item);

                    archiver.download_one(&item).await
                }
            })
            .await;

        println!("Result: {:?}", archiver.stats());

        result
    }
}

#[derive(Debug, StructOpt)]
struct Download {
    url: String,
    output: PathBuf,
}

impl Download {
    async fn run(&self, _: PhotosLibrary) -> Result<()> {
        let client = reqwest::Client::new();
        downloader::download(&client, self.url.as_str(), &self.output).await?;
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
enum Cmd {
    List(List),
    Archive(Archive),
    Download(Download),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "photo-archiver", about = "Archiver tool for google photos")]
struct Args {
    #[structopt(flatten)]
    auth: Auth,

    #[structopt(subcommand)]
    cmd: Cmd,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::from_args();

    let authenticator = args.auth.authenticator().await?;

    let https = hyper_rustls::HttpsConnector::with_native_roots();
    let client = hyper::Client::builder().build(https);

    let hub = PhotosLibrary::new(client, authenticator);

    match args.cmd {
        Cmd::List(list) => list.run(hub).await?,
        Cmd::Archive(archive) => archive.run(hub).await?,
        Cmd::Download(download) => download.run(hub).await?,
    }

    Ok(())
}
