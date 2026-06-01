//! Generates the genesis fixtures (`.mac` account files + `genesis.toml`) used to bootstrap a
//! testing node from the standalone node executables.
//!
//! Usage: `gen-genesis [OUTPUT_DIR]` (defaults to `./genesis`).

use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let output_dir = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from("./genesis"), PathBuf::from);

    node_builder::write_genesis_config(&output_dir)?;
    println!("Wrote genesis config to {}", output_dir.display());

    Ok(())
}
