use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use clap::Parser;
use miden_client::account::AccountId;
use miden_client::assembly::CodeBuilder;
use miden_client::keystore::Keystore;
use miden_client::rpc::domain::account::AccountStorageRequirements;
use miden_client::transaction::{
    AdviceInputs,
    ForeignAccount,
    TransactionRequestBuilder,
    TransactionRequestError,
    TransactionScript,
};
use miden_client::vm::{Package, PackageExport};
use miden_client::{Client, Deserializable, Felt, Word};

use crate::errors::CliError;
use crate::utils::{parse_account_id, print_executed_program_stack, print_executed_transaction};

// CALL COMMAND
// ================================================================================================

#[derive(Debug, Clone, Parser)]
#[command(about = "Call a procedure on a local account and display the result and state delta")]
pub struct CallCmd {
    /// Account and procedure in the form `<ACCOUNT_ID>:<PROCEDURE>`.
    #[arg(value_name = "ACCOUNT_ID:PROCEDURE")]
    target: String,

    /// Positional arguments to push onto the stack before calling the procedure.
    #[arg(value_name = "args")]
    args: Vec<String>,

    /// Path to the package (.masp) file containing the procedure. If omitted, `<PROCEDURE>` must
    /// be a hex digest and the output stack is shown as raw felts.
    #[arg(long, short)]
    package: Option<PathBuf>,
}

impl CallCmd {
    pub async fn execute<AUTH: Keystore + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
    ) -> Result<(), CliError> {
        if client.get_sync_height().await? == 0.into() {
            return Err(CliError::InvalidArgument(
                "Client has not been synced yet. Run `miden-client sync` first.".to_string(),
            ));
        }

        let (account_str, procedure) = self.target.split_once(':').ok_or_else(|| {
            CliError::InvalidArgument(format!(
                "Expected `<ACCOUNT_ID>:<PROCEDURE>`, got '{}'.",
                self.target
            ))
        })?;

        let target_id = parse_account_id(&client, account_str).await?;
        let call_target = resolve_call_target(&client, target_id).await?;
        let args = parse_args(&self.args)?;
        let call_code = self.resolve_call_code(&client, procedure, &args)?;

        if call_target.is_remote() {
            run_remote_call(&mut client, &call_target, target_id, call_code, &args).await
        } else {
            run_local_call(&mut client, call_target.executor, call_code, &args).await
        }
    }

    /// Resolves the procedure digest and code builder either from `--package` (calling by name)
    /// or from a hex digest when no package is given.
    fn resolve_call_code<AUTH: Keystore + Sync + 'static>(
        &self,
        client: &Client<AUTH>,
        procedure: &str,
        args: &[Felt],
    ) -> Result<CallCode, CliError> {
        if let Some(pkg_path) = &self.package {
            let package = load_package(pkg_path)?;
            let digest = resolve_procedure_digest(&package, procedure)?;
            let ProcedureSignature { param_count, result_count } =
                print_manifest_signature(&package, procedure);

            match param_count {
                Some(expected) if args.len() != expected => {
                    return Err(CliError::InvalidArgument(format!(
                        "Procedure '{procedure}' expects {expected} argument(s), got {}.",
                        args.len()
                    )));
                },
                None => {
                    println!(
                        "Warning: no type info for procedure '{procedure}'. Skipping \
                         argument count check. Passing a wrong number of arguments may \
                         cause errors or wrong results."
                    );
                },
                _ => {},
            }

            // Dynamic linking lets the assembler resolve `call.<digest>` without embedding
            // the library bytes in the script.
            let builder =
                client.code_builder().with_dynamically_linked_library(package.mast.as_ref())?;
            Ok(CallCode { builder, digest, result_count })
        } else {
            let digest = Word::try_from(procedure).map_err(|_| {
                CliError::InvalidArgument(format!(
                    "'{procedure}' is not a hex digest. Pass `--package <FILE>.masp` to \
                     call a procedure by name, or give its hex digest to call without a \
                     package."
                ))
            })?;
            println!(
                "No `--package` provided; output will be raw felts. Pass \
                 `--package <FILE>.masp` for typed output."
            );
            Ok(CallCode {
                builder: client.code_builder(),
                digest,
                result_count: None,
            })
        }
    }
}

// HELPERS
// ================================================================================================

/// Resolved call code: the linked builder, the procedure digest, and the result count when known.
struct CallCode {
    builder: CodeBuilder,
    digest: Word,
    result_count: Option<usize>,
}

/// Runs a remote call via FPI. FPI cannot mutate the foreign account, so there is no state delta
/// to compute — only the read phase runs.
async fn run_remote_call<AUTH: Keystore + Sync + 'static>(
    client: &mut Client<AUTH>,
    call_target: &CallTarget,
    target_id: AccountId,
    call_code: CallCode,
    args: &[Felt],
) -> Result<(), CliError> {
    let CallCode { builder, digest, result_count } = call_code;
    let tx_script = generate_fpi_tx_script(builder, target_id, &digest, args)?;

    let output_stack = client
        .execute_program(
            call_target.executor,
            tx_script,
            AdviceInputs::default(),
            call_target.foreign_accounts.clone(),
        )
        .await?;

    print_executed_program_stack(&output_stack, result_count);
    println!("\nRemote calls are read-only; no state delta.");
    Ok(())
}

/// Runs a local call: a read phase for the return values, then a transaction for the state delta.
/// The executor is the target account itself, so the procedure may mutate it.
async fn run_local_call<AUTH: Keystore + Sync + 'static>(
    client: &mut Client<AUTH>,
    executor: AccountId,
    call_code: CallCode,
    args: &[Felt],
) -> Result<(), CliError> {
    let CallCode { builder, digest, result_count } = call_code;
    let read_tx_script = generate_tx_script(builder.clone(), &digest, args, result_count)?;
    let delta_tx_script = generate_tx_script(builder, &digest, args, Some(0))?;

    // 1) Read-only execution to get return values.
    let output_stack = client
        .execute_program(executor, read_tx_script, AdviceInputs::default(), BTreeMap::new())
        .await?;
    print_executed_program_stack(&output_stack, result_count);

    // 2) Transaction execution to get the state delta.
    let tx_request = TransactionRequestBuilder::new()
        .custom_script(delta_tx_script)
        .build()
        .map_err(|err| {
            CliError::Transaction(err.into(), "Failed to build transaction".to_string())
        })?;

    match client.execute_transaction(executor, tx_request).await {
        Ok(tx_result) => print_executed_transaction(tx_result.executed_transaction())?,
        Err(e) => println!("\n(Could not compute state delta: {e})"),
    }
    Ok(())
}

/// Resolved call target. Local accounts run themselves; remote accounts are read via FPI
/// using a local account as executor.
struct CallTarget {
    executor: AccountId,
    foreign_accounts: BTreeMap<AccountId, ForeignAccount>,
}

impl CallTarget {
    fn is_remote(&self) -> bool {
        !self.foreign_accounts.is_empty()
    }
}

async fn resolve_call_target<AUTH: Keystore + Sync + 'static>(
    client: &Client<AUTH>,
    target_id: AccountId,
) -> Result<CallTarget, CliError> {
    if client.get_account(target_id).await?.is_some() {
        return Ok(CallTarget {
            executor: target_id,
            foreign_accounts: BTreeMap::new(),
        });
    }

    let executor = pick_local_executor(client).await?;

    let foreign_account = ForeignAccount::public(target_id, AccountStorageRequirements::default())
        .map_err(|err| match err {
            TransactionRequestError::InvalidForeignAccountId(_) => {
                CliError::InvalidArgument(format!(
                    "Account {target_id} is not in the local store and is not a public account; \
                     remote calls require an account with public state."
                ))
            },
            other => CliError::InvalidArgument(format!(
                "Failed to construct foreign account for {target_id}: {other}"
            )),
        })?;

    println!(
        "Account {target_id} not found locally; reading from network via FPI \
         (executor: {executor})."
    );

    Ok(CallTarget {
        executor,
        foreign_accounts: BTreeMap::from([(target_id, foreign_account)]),
    })
}

/// Picks the first local regular account to use as the FPI executor.
async fn pick_local_executor<AUTH: Keystore + Sync + 'static>(
    client: &Client<AUTH>,
) -> Result<AccountId, CliError> {
    let headers = client.get_account_headers().await?;
    headers
        .into_iter()
        .map(|(header, _status)| header.id())
        .find(|id| id.account_type().is_regular_account())
        .ok_or_else(|| {
            CliError::InvalidArgument(
                "No local regular account found to make the remote call from. Create one with \
                 `miden-client new-wallet`/`new-account`) and re-run."
                    .to_string(),
            )
        })
}

fn load_package(path: &Path) -> Result<Package, CliError> {
    if !path.exists() {
        return Err(CliError::InvalidArgument(format!(
            "Package file not found: {}",
            path.display()
        )));
    }
    let bytes = std::fs::read(path)?;
    Package::read_from_bytes(&bytes).map_err(|e| {
        CliError::Parse(Box::new(e), format!("Failed to deserialize package: {}", path.display()))
    })
}

fn resolve_procedure_digest(package: &Package, procedure_name: &str) -> Result<Word, CliError> {
    let library = &*package.mast;
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

fn parse_args(args: &[String]) -> Result<Vec<Felt>, CliError> {
    args.iter()
        .map(|arg| {
            let n = arg.parse::<u64>().map_err(|_| {
                CliError::InvalidArgument(format!("Invalid argument '{arg}'. Expected u64."))
            })?;
            Felt::try_from(n)
                .map_err(|_| CliError::InvalidArgument(format!("Argument '{arg}' is too large.")))
        })
        .collect()
}

/// Parameter and result counts from a procedure's manifest signature. `None` means the
/// information is unavailable (procedure missing from manifest or export lacks type info).
struct ProcedureSignature {
    param_count: Option<usize>,
    result_count: Option<usize>,
}

/// Prints the signature of `procedure_name` from the package manifest and returns its parameter
/// and result counts. If the procedure is missing, prints the list of available exports.
fn print_manifest_signature(package: &Package, procedure_name: &str) -> ProcedureSignature {
    const UNKNOWN: ProcedureSignature =
        ProcedureSignature { param_count: None, result_count: None };

    let kebab_name = procedure_name.replace('_', "-");
    let quoted_kebab = format!("\"{kebab_name}\"");
    let quoted_name = format!("\"{procedure_name}\"");

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

        if let Some(sig) = &proc_export.signature {
            let params: Vec<String> = sig.params.iter().map(|p| format!("{p:?}")).collect();
            let results: Vec<String> = sig.results.iter().map(|r| format!("{r:?}")).collect();

            let ret_str = if results.is_empty() {
                String::new()
            } else {
                format!(" -> ({})", results.join(", "))
            };

            let params_str = params.join(", ");
            println!("Raw Signature: {procedure_name}({params_str}){ret_str}\n");

            return ProcedureSignature {
                param_count: Some(sig.params.len()),
                result_count: Some(sig.results.len()),
            };
        }
        println!("Raw Signature: {procedure_name}(...) [no type info]\n");
        return UNKNOWN;
    }

    println!("(procedure '{procedure_name}' not found in manifest exports)");
    println!("Available exports:");
    for export in package.manifest.exports() {
        if let PackageExport::Procedure(p) = export {
            println!("  {}", p.path);
        }
    }
    println!();
    UNKNOWN
}

/// Builds a transaction script that pushes `args`, calls the procedure at `digest`, and optionally
/// drops the pushed args from under the results. `Some(n)` keeps the top `n` values; `None` skips
/// drops.
fn generate_tx_script(
    code_builder: CodeBuilder,
    digest: &Word,
    args: &[Felt],
    result_count: Option<usize>,
) -> Result<TransactionScript, CliError> {
    // MASM `movup.n` only works for n in 2..=15. The VM stack exposes only the top
    // 16 elements; anything deeper lives in the overflow table and cannot be reached
    // by `movup`. So we can't drop args from under more than 15 results.
    // See miden-vm/docs/src/user_docs/assembly/instruction_reference.md (movup row)
    // and miden-vm/docs/src/design/stack/stack_ops.md (MOVUP/MOVDN sections).
    if let Some(n) = result_count
        && n > 15
    {
        return Err(CliError::InvalidArgument(format!(
            "Procedure returns {n} values; only up to 15 are supported."
        )));
    }

    let mut script = String::from("use miden::core::sys\n\nbegin\n");

    // Push args in reverse so the first arg ends up on top.
    for arg in args.iter().rev() {
        writeln!(script, "    push.{arg}").unwrap();
    }

    writeln!(script, "    call.{}", digest.to_hex()).unwrap();

    let to_drop = args.len();
    if to_drop > 0 {
        match result_count {
            Some(0) => {
                for _ in 0..to_drop {
                    script.push_str("    drop\n");
                }
            },
            Some(1) => {
                for _ in 0..to_drop {
                    script.push_str("    swap drop\n");
                }
            },
            Some(n) => {
                for _ in 0..to_drop {
                    writeln!(script, "    movup.{n} drop").unwrap();
                }
            },
            None => {},
        }
    }

    // Without a known result count we can't drop the pushed args from under the results, so the
    // stack ends up with extra elements. `truncate_stack` enforces the 16-element exit invariant.
    script.push_str("    exec.sys::truncate_stack\n");
    script.push_str("end\n");
    Ok(code_builder.compile_tx_script(&script)?)
}

/// Builds a script that invokes `proc_digest` on `foreign_id` via FPI. Args are pushed so
/// args[0] ends up on top, matching the direct-call convention. `truncate_stack` enforces the
/// 16-element exit invariant required by FPI component exports.
fn generate_fpi_tx_script(
    code_builder: CodeBuilder,
    foreign_id: AccountId,
    proc_digest: &Word,
    args: &[Felt],
) -> Result<TransactionScript, CliError> {
    const FPI_INPUT_SLOTS: usize = 16;
    if args.len() > FPI_INPUT_SLOTS {
        return Err(CliError::InvalidArgument(format!(
            "FPI supports up to {FPI_INPUT_SLOTS} input felts; got {}",
            args.len()
        )));
    }

    let mut script = String::from("use miden::protocol::tx\nuse miden::core::sys\n\nbegin\n");

    // Pad the deeper input slots with zeros, then push args so args[0] lands on top.
    let pad_count = FPI_INPUT_SLOTS - args.len();
    let full_words = pad_count / 4;
    let remainder = pad_count % 4;
    for _ in 0..full_words {
        script.push_str("    padw\n");
    }
    for _ in 0..remainder {
        script.push_str("    push.0\n");
    }
    for arg in args.iter().rev() {
        writeln!(script, "    push.{arg}").unwrap();
    }

    writeln!(script, "    push.{}", proc_digest.to_hex()).unwrap();
    writeln!(script, "    push.{}", foreign_id.prefix().as_u64()).unwrap();
    writeln!(script, "    push.{}", foreign_id.suffix()).unwrap();

    script.push_str("    exec.tx::execute_foreign_procedure\n");
    script.push_str("    exec.sys::truncate_stack\n");
    script.push_str("end\n");

    Ok(code_builder.compile_tx_script(&script)?)
}
