use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};

use miden_client::account::AccountType;
use miden_client::account::component::{
    AccountComponentMetadata,
    AuthMultisig,
    AuthSingleSig,
    AuthSingleSigAcl,
    BasicFungibleFaucet,
    BasicWallet,
    MIDEN_PACKAGE_EXTENSION,
    NoAuth,
    SchemaType,
    StorageSchema,
    StorageSlotSchema,
    basic_fungible_faucet_library,
    basic_wallet_library,
    multisig_library,
    no_auth_library,
    singlesig_acl_library,
    singlesig_library,
};
use miden_client::assembly::Library;
use miden_client::utils::Serializable;
use miden_client::vm::{
    MastArtifact,
    Package,
    PackageExport,
    PackageKind,
    PackageManifest,
    ProcedureExport,
    QualifiedProcedureName,
    Section,
    SectionId,
};

const PACKAGE_DIR: &str = "packages";

fn main() {
    // Basic wallet (no storage schema)
    let basic_wallet_metadata = BasicWallet::component_metadata();
    build_package("basic-wallet", basic_wallet_library(), &basic_wallet_metadata, None);

    // Basic fungible faucet
    let basic_faucet_metadata = BasicFungibleFaucet::component_metadata();
    build_package(
        "basic-fungible-faucet",
        basic_fungible_faucet_library(),
        &basic_faucet_metadata,
        None,
    );

    // Basic auth (singlesig - supports both RPO Falcon and ECDSA)
    let singlesig_metadata = AuthSingleSig::component_metadata();

    build_package("basic-auth", singlesig_library(), &singlesig_metadata, Some("auth"));

    // ECDSA auth (same component, different package name for discoverability)
    build_package("ecdsa-auth", singlesig_library(), &singlesig_metadata, Some("auth"));

    // No authentication component. Nonce is incremented on first transaction and when the account
    // state is changed. Provides no cryptographic authentication.
    let no_auth_metadata = NoAuth::component_metadata();
    build_package("no-auth", no_auth_library(), &no_auth_metadata, Some("auth"));

    // Multisig auth
    //
    // NOTE: We use a custom schema for `approver_schemes` because the upstream
    // `approver_auth_scheme_slot_schema()` defines the map value as
    // `SchemaType::auth_scheme()` (a felt type), but the type registry expects
    // felt-types-as-words in the format `[0, 0, 0, <felt>]` while the actual
    // storage uses `[felt, 0, 0, 0]`. Using `native_word()` avoids this
    // validation mismatch.
    let approver_schemes_schema = (
        AuthMultisig::approver_scheme_ids_slot().clone(),
        StorageSlotSchema::map("Approver scheme IDs", SchemaType::u32(), SchemaType::native_word()),
    );
    let multisig_metadata = AccountComponentMetadata::new(AuthMultisig::NAME, AccountType::all())
        .with_description("Multisig authentication component using hybrid signature schemes")
        .with_storage_schema(
            StorageSchema::new([
                AuthMultisig::threshold_config_slot_schema(),
                AuthMultisig::approver_public_keys_slot_schema(),
                approver_schemes_schema,
                AuthMultisig::executed_transactions_slot_schema(),
                AuthMultisig::procedure_thresholds_slot_schema(),
            ])
            .expect("storage schema should be valid"),
        );
    build_package("multisig-auth", multisig_library(), &multisig_metadata, Some("auth"));

    // ACL auth
    let acl_metadata = AuthSingleSigAcl::component_metadata();
    build_package("acl-auth", singlesig_acl_library(), &acl_metadata, Some("auth"));
}

/// Builds a package and stores it under `{OUT_DIR}/{PACKAGE_DIR}` or
/// `{OUT_DIR}/{PACKAGE_DIR}/{subdirectory}` if a subdirectory is provided.
pub fn build_package(
    package_name: &str,
    library: Library,
    metadata: &AccountComponentMetadata,
    subdirectory: Option<&str>,
) {
    // NOTE: Taken from the miden-compiler's build_package function:
    // https://github.com/0xMiden/compiler/blob/61ee77f57c07c197323728642f8feca972b24217/midenc-compile/src/stages/assemble.rs#L71-L88
    // Gather all of the procedure metadata for exports of this package
    let mut exports: Vec<PackageExport> = Vec::new();
    for module_info in library.module_infos() {
        for (_, proc_info) in module_info.procedures() {
            let name = QualifiedProcedureName::new(module_info.path(), proc_info.name.clone());
            let export = ProcedureExport {
                path: name.into_inner(),
                digest: proc_info.digest,
                signature: proc_info.signature.as_deref().cloned(),
                attributes: proc_info.attributes.clone(),
            };
            exports.push(PackageExport::Procedure(export));
        }
    }

    let mast = MastArtifact::Library(Arc::new(library));

    let manifest = PackageManifest::new(exports);

    let account_component_metadata_section =
        Section::new(SectionId::ACCOUNT_COMPONENT_METADATA, metadata.to_bytes());

    let package = Package {
        name: metadata.name().to_string(),
        version: Some(metadata.version().clone()),
        description: Some(metadata.description().to_string()),
        mast,
        manifest,
        sections: vec![account_component_metadata_section],
        kind: PackageKind::AccountComponent,
    };

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

    // Write the file
    let mut packages_out_dir = PathBuf::from(&out_dir).join(PACKAGE_DIR);
    if let Some(subdir) = subdirectory {
        packages_out_dir = packages_out_dir.join(subdir);
    }
    fs::create_dir_all(&packages_out_dir).expect("Failed to packages directory in OUT_DIR");

    let output_filename = format!("{package_name}.{MIDEN_PACKAGE_EXTENSION}");
    let output_file = packages_out_dir.join(&output_filename);

    fs::write(&output_file, package.to_bytes()).unwrap_or_else(|e| {
        panic!(
            "Failed to write Package {} to file {} in {}. Error: {}",
            package.name, output_filename, out_dir, e
        );
    });
}
