//! Generates the genesis fixtures (`.mac` account files + `genesis.toml`) used to bootstrap a
//! testing node from the standalone node executables.
//!
//! Usage: `gen-genesis [OUTPUT_DIR]` (defaults to `./genesis`).
//!
//! Setting the `AGGLAYER_GENESIS` env var additionally emits the agglayer genesis accounts
//! (bridge admin, GER manager, bridge, and faucet).

use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let output_dir = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from("./genesis"), PathBuf::from);

    let include_agglayer = std::env::var("AGGLAYER_GENESIS").is_ok();
    if include_agglayer {
        println!("Agglayer genesis accounts enabled");
    }

    test_node_genesis::write_genesis_config(&output_dir, include_agglayer)?;
    println!("Wrote genesis config to {}", output_dir.display());

    Ok(())
}
