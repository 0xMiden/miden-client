use clap::Parser;

#[derive(Debug, Parser)]
pub struct InfoCmd {
    /// Display detailed RPC node status information.
    #[arg(short = 'r', long = "rpc-status")]
    pub rpc_status: bool,
}
