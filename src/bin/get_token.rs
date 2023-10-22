use eyre::{Result, WrapErr};
use structopt::StructOpt;
use yup_oauth2::{
    noninteractive::NoninteractiveTokens, InstalledFlowAuthenticator, InstalledFlowReturnMethod,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "get-token", about = "Get token for the given scopes")]
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

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .build()
        .await
        .wrap_err("Failed to build authenticator")?;

    let tokens = NoninteractiveTokens::builder(&auth)?
        .add_token_for(&opt.scopes, false)
        .await
        .wrap_err("Failed to get token")?
        .build();

    println!("Tokens: {:?}", tokens);

    let serialized = serde_json::to_string_pretty(&tokens).wrap_err("Failed to serialize")?;
    std::fs::write(opt.output, serialized).wrap_err("Failed to write access token to file")?;

    Ok(())
}
