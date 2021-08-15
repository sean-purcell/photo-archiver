use eyre::{Result, WrapErr};
use futures::stream::{StreamExt, TryStreamExt};
use google_photoslibrary1::PhotosLibrary;
use structopt::StructOpt;
use yup_oauth2::{
    authenticator::DefaultAuthenticator, noninteractive::NoninteractiveTokens,
    NoninteractiveAuthenticator,
};

mod media_item_iter;

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

#[derive(Debug, StructOpt)]
struct List {
    #[structopt(
        short = "n",
        long = "num",
        help = "Number of items to list",
        default_value = "50"
    )]
    num: usize,
}

impl List {
    async fn run(&self, hub: PhotosLibrary) -> Result<()> {
        let iter = media_item_iter::list(hub);
        iter.take(self.num)
            .enumerate()
            .map(|(i, val)| val.map(|item| (i, item)))
            .try_for_each(|(i, item)| async move {
                println!("{}: {:?}", i, item);
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
    root_dir: String,
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
