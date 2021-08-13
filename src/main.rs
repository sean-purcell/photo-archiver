use eyre::{Result, WrapErr};
use structopt::StructOpt;
use yup_oauth2::AccessToken;

#[derive(Debug, StructOpt)]
struct Auth {
    #[structopt(short = "t", long = "token")]
    token: String,
}

impl Auth {
    fn access_token(&self) -> Result<AccessToken> {
        let contents =
            std::fs::read(&self.token).wrap_err(format!("Couldn't load file: {}", self.token))?;
        let token = serde_json::from_slice::<AccessToken>(&contents)
            .wrap_err(format!("Failed to parse token file: {}", self.token))?;
        Ok(token)
    }
}

#[derive(Debug, StructOpt)]
enum Cmd {
    List,
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

    let token = args.auth.access_token()?;

    match args.cmd {}

    Ok(())
}
