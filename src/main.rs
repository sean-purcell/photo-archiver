use std::path::PathBuf;

use eyre::{Result, WrapErr};
use futures::stream::{StreamExt, TryStreamExt};
use google_photoslibrary1::PhotosLibrary;
use structopt::StructOpt;
use yup_oauth2::{
    authenticator::DefaultAuthenticator, noninteractive::NoninteractiveTokens,
    NoninteractiveAuthenticator,
};

mod item;
mod media_item_iter;

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
        help = "Print style for item (options: debug, fspath)",
        default_value = "debug",
        parse(try_from_str = Style::parse),
    )]
    style: Style,
}

impl List {
    async fn run(&self, hub: PhotosLibrary) -> Result<()> {
        let iter = media_item_iter::list(hub);
        iter.take(self.num)
            .enumerate()
            .map(|(i, val)| val.map(|item| (i, item)))
            .try_for_each(|(i, item)| async move {
                let rep = self.style.serialize(&(item.into()));
                println!("{}: {}", i, rep);
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
}

impl Archive {
    async fn run(&self, _: PhotosLibrary) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
enum Cmd {
    List(List),
    Archive(Archive),
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
    let args = Args::from_args();

    let authenticator = args.auth.authenticator().await?;

    let https = hyper_rustls::HttpsConnector::with_native_roots();
    let client = hyper::Client::builder().build(https);

    let hub = PhotosLibrary::new(client, authenticator);

    match args.cmd {
        Cmd::List(list) => list.run(hub).await?,
        Cmd::Archive(archive) => archive.run(hub).await?,
    }

    Ok(())
}
