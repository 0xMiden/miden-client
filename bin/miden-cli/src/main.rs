use clap::FromArgMatches;
use miden_client_cli::{Cli, MidenClientCli};

extern crate std;

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();

    // read command-line args
    let input = <MidenClientCli as clap::CommandFactory>::command();
    let matches = input.get_matches();
    let parsed = MidenClientCli::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());
    let cli: Cli = parsed.into();

    // execute cli action
    Ok(cli.execute().await?)
}
