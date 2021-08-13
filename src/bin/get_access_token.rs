use eyre::{Result, WrapErr};
use structopt::StructOpt;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "get-access-token",
    about = "Get access token for the given scopes"
)]
struct Opt {
    #[structopt(short = "c", long = "client-secret")]
    client_secret: String,
    #[structopt(
        short = "s",
        long = "scope",
        default_value = "https://www.googleapis.com/auth/photoslibrary.readonly"
    )]
    scopes: Vec<String>,
    #[structopt(short = "o", long = "output")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    println!("Options: {:?}", opt);

    let secret = yup_oauth2::read_application_secret(opt.client_secret)
        .await
        .wrap_err("Failed to read application secret")?;

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::Interactive)
        .build()
        .await
        .wrap_err("Failed to build authenticator")?;

    let token = auth
        .token(&opt.scopes)
        .await
        .wrap_err("Failed to get token")?;

    println!("Token: {:?}", token);

    let serialized = serde_json::to_string_pretty(&token).wrap_err("Failed to serialize")?;
    std::fs::write(opt.output, serialized).wrap_err("Failed to write access token to file")?;

    Ok(())
}
