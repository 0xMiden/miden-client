use std::collections::BTreeSet;
use std::path::PathBuf;

use clap::Parser;
use miden_client::keystore::Keystore;
use miden_client::transaction::{AdviceInputs, TransactionRequestBuilder};
use miden_client::vm::Package;
use miden_client::{Client, Deserializable, Felt, Word};

use crate::create_dynamic_table;
use crate::errors::CliError;
use crate::utils::parse_account_id;

// CALL COMMAND
// ================================================================================================

#[derive(Debug, Clone, Parser)]
#[command(about = "Call a procedure on a local account and display the result and state delta")]
pub struct CallCmd {
    /// Account ID and procedure name in the format `<ACCOUNT_ID>::<PROCEDURE_NAME>`.
    #[arg(value_name = "account_id::procedure_name")]
    target: String,

    /// Positional arguments to push onto the stack before calling the procedure.
    #[arg(value_name = "args")]
    args: Vec<String>,

    /// Path to the package (.masp) file containing the procedure.
    #[arg(long, short)]
    package: PathBuf,
}

impl CallCmd {
    pub async fn execute<AUTH: Keystore + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
    ) -> Result<(), CliError> {
        // Sync state to ensure we have the latest block data
        client.sync_state().await?;

        // Parse target
        let (account_id_str, procedure_name) = parse_target(&self.target)?;
        let account_id = parse_account_id(&client, &account_id_str).await?;

        // Load the package and resolve the procedure
        let package = load_package(&self.package)?;
        let digest = resolve_procedure_digest(&package, &procedure_name)?;

        // Print signature and determine result count
        let result_count = print_manifest_signature(&package, &procedure_name);

        // Parse arguments
        let args = parse_args(&self.args)?;

        let library = package.unwrap_library();

        // 1) Read-only execution to get return values
        let read_script = generate_tx_script(&digest, &args, result_count);

        let read_tx_script = client
            .code_builder()
            .with_statically_linked_library(&library)?
            .compile_tx_script(&read_script)?;

        let output_stack = client
            .execute_program(account_id, read_tx_script, AdviceInputs::default(), BTreeSet::new())
            .await?;

        // Print result
        print_output_stack(&output_stack, result_count);

        // 2) Transaction execution to get state delta
        let delta_script = generate_tx_script(&digest, &args, 0);
        let delta_tx_script = client
            .code_builder()
            .with_statically_linked_library(&library)?
            .compile_tx_script(&delta_script)?;

        let tx_request = TransactionRequestBuilder::new()
            .custom_script(delta_tx_script)
            .build()
            .map_err(|err| {
                CliError::Transaction(err.into(), "Failed to build transaction".to_string())
            })?;

        match client.execute_transaction(account_id, tx_request).await {
            Ok(tx_result) => {
                print_delta(tx_result.executed_transaction().account_delta());
            },
            Err(e) => {
                println!("\n(Could not compute state delta: {e})");
            },
        }

        Ok(())
    }
}

// HELPERS
// ================================================================================================

fn parse_target(target: &str) -> Result<(String, String), CliError> {
    let parts: Vec<&str> = target.splitn(2, "::").collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(CliError::InvalidArgument(format!(
            "Invalid target '{}'. Expected ACCOUNT_ID::PROCEDURE_NAME",
            target
        )));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn load_package(path: &PathBuf) -> Result<Package, CliError> {
    if !path.exists() {
        return Err(CliError::InvalidArgument(format!(
            "Package file not found: {}",
            path.display()
        )));
    }
    let bytes = std::fs::read(path)
        .map_err(|e| CliError::Exec(Box::new(e), format!("Failed to read: {}", path.display())))?;
    Package::read_from_bytes(&bytes).map_err(|e| {
        CliError::Exec(Box::new(e), format!("Failed to deserialize: {}", path.display()))
    })
}

fn resolve_procedure_digest(package: &Package, procedure_name: &str) -> Result<Word, CliError> {
    if !package.is_library() {
        return Err(CliError::InvalidArgument("Package does not contain a library.".to_string()));
    }

    let library = package.unwrap_library();
    for module_info in library.module_infos() {
        if let Some(digest) = module_info.get_procedure_digest_by_name(procedure_name) {
            return Ok(digest);
        }
    }

    let mut available = Vec::new();
    for module_info in library.module_infos() {
        for (_idx, proc_info) in module_info.procedures() {
            available.push(format!("  {}::{}", module_info.path(), proc_info.name));
        }
    }
    Err(CliError::InvalidArgument(format!(
        "Procedure '{}' not found. Available:\n{}",
        procedure_name,
        available.join("\n")
    )))
}

fn parse_args(args: &[String]) -> Result<Vec<u64>, CliError> {
    args.iter()
        .map(|arg| {
            arg.parse::<u64>().map_err(|_| {
                CliError::InvalidArgument(format!("Invalid argument '{}'. Expected u64.", arg))
            })
        })
        .collect()
}

fn print_manifest_signature(package: &Package, procedure_name: &str) -> usize {
    use miden_client::vm::PackageExport;

    let kebab_name = procedure_name.replace('_', "-");
    let quoted_kebab = format!("\"{}\"", kebab_name);
    let quoted_name = format!("\"{}\"", procedure_name);

    for export in package.manifest.exports() {
        let PackageExport::Procedure(proc_export) = export else {
            continue;
        };

        let path_str = proc_export.path.to_string();
        if !path_str.ends_with(&kebab_name)
            && !path_str.ends_with(procedure_name)
            && !path_str.ends_with(&quoted_kebab)
            && !path_str.ends_with(&quoted_name)
        {
            continue;
        }

        match &proc_export.signature {
            Some(sig) => {
                let params: Vec<String> = sig.params.iter().map(|p| format!("{p:?}")).collect();
                let results: Vec<String> = sig.results.iter().map(|r| format!("{r:?}")).collect();

                let ret_str = if results.is_empty() {
                    String::new()
                } else {
                    format!(" -> ({})", results.join(", "))
                };

                println!("Raw Signature: {}({}){}\n", procedure_name, params.join(", "), ret_str);

                return sig.results.len();
            },
            None => {
                println!("Raw Signature: {}(...) [no type info]\n", procedure_name);
                return 0;
            },
        }
    }

    println!("(procedure '{}' not found in manifest exports)", procedure_name);
    println!("Available exports:");
    for export in package.manifest.exports() {
        if let PackageExport::Procedure(p) = export {
            println!("  {}", p.path);
        }
    }
    println!();
    0
}

fn print_output_stack(stack: &[Felt; 16], expected_results: usize) {
    let count = if expected_results > 0 {
        expected_results
    } else {
        stack.iter().rposition(|v| v.as_int() != 0).map(|pos| pos + 1).unwrap_or(0)
    };

    if count == 0 {
        println!("\nResult: 0");
    } else if count == 1 {
        println!("\nResult: {}", stack[0]);
    } else {
        println!("\nResult ({count} values):");
        for i in 0..count {
            println!("  [{i}]: {}", stack[i]);
        }
    }
}

fn generate_tx_script(digest: &Word, args: &[u64], result_count: usize) -> String {
    let mut script = String::from("begin\n");

    // Push args in reverse so first arg ends up on top
    for arg in args.iter().rev() {
        script.push_str(&format!("    push.{arg}\n"));
    }

    script.push_str(&format!("    call.{}\n", digest.to_hex()));

    // Drop pushed args from under the results to restore stack depth to 16
    let to_drop: usize = args.len();
    if to_drop > 0 {
        match result_count {
            0 => {
                for _ in 0..to_drop {
                    script.push_str("    drop\n");
                }
            },
            1 => {
                for _ in 0..to_drop {
                    script.push_str("    swap drop\n");
                }
            },
            n => {
                for _ in 0..to_drop {
                    script.push_str(&format!("    movup.{n} drop\n"));
                }
            },
        }
    }

    script.push_str("end\n");
    script
}

fn print_delta(delta: &miden_client::account::AccountDelta) {
    let has_values = delta.storage().values().next().is_some();
    let has_maps = delta.storage().maps().next().is_some();
    let has_vault = !delta.vault().is_empty();

    if !has_values && !has_maps && !has_vault {
        println!("\nState delta: no changes");
        return;
    }

    println!("\nState delta:");

    if has_values {
        let mut table = create_dynamic_table(&["Storage Slot", "New Value"]);
        for (slot, value) in delta.storage().values() {
            table.add_row(vec![slot.to_string(), value.to_hex()]);
        }
        println!("{table}");
    }

    if has_maps {
        let mut table = create_dynamic_table(&["Storage Slot", "Map Key", "New Value"]);
        for (slot, map_delta) in delta.storage().maps() {
            for (key, value) in map_delta.entries() {
                table.add_row(vec![slot.to_string(), key.inner().to_hex(), value.to_hex()]);
            }
        }
        println!("{table}");
    }

    if has_vault {
        println!("Vault changes detected.");
    }

    println!("Nonce delta: {}", delta.nonce_delta());
}
