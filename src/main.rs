use eyre::{Result, WrapErr};
use google_photoslibrary1::PhotosLibrary;
use structopt::StructOpt;
use yup_oauth2::{
    authenticator::DefaultAuthenticator, noninteractive::NoninteractiveTokens,
    NoninteractiveAuthenticator,
};

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
enum Cmd {
    List {
        #[structopt(
            short = "n",
            long = "num",
            help = "Number of items to list",
            default_value = "50"
        )]
        num: u64,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(name = "photo-archiver", about = "Archiver tool for google photos")]
struct Args {
    #[structopt(flatten)]
    auth: Auth,

    #[structopt(subcommand)]
    cmd: Cmd,
}

async fn list(hub: PhotosLibrary, num: u64) -> Result<()> {
    let mut fetched = 0u64;
    let mut page_token: Option<String> = None;
    while fetched < num {
        let req = hub.media_items().list().page_size(100);
        let req = match page_token {
            Some(token) => req.page_token(token.as_str()),
            None => req,
        };
        let (_body, response) = req.doit().await.wrap_err("Failed to list items")?;

        for item in response.media_items.unwrap_or_else(|| vec![]).iter() {
            if fetched < num {
                println!("{}: {:?}", fetched, item);
            }
            fetched += 1;
        }

        match response.next_page_token {
            Some(token) => page_token = Some(token),
            None => break,
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    let authenticator = args.auth.authenticator().await?;

    let https = hyper_rustls::HttpsConnector::with_native_roots();
    let client = hyper::Client::builder().build(https);

    let hub = PhotosLibrary::new(client, authenticator);

    match args.cmd {
        Cmd::List { num } => list(hub, num).await?,
    }

    Ok(())
}
