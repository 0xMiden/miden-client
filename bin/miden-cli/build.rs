use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs};

use miden_client::account::component::{
    AccountComponentMetadata,
    MIDEN_PACKAGE_EXTENSION,
    basic_fungible_faucet_library,
    basic_wallet_library,
    ecdsa_k256_keccak_library,
    no_auth_library,
    rpo_falcon_512_acl_library,
    rpo_falcon_512_library,
    rpo_falcon_512_multisig_library,
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
    build_package(&PathBuf::from("templates/basic-wallet.toml"), basic_wallet_library(), None);

    build_package(
        &PathBuf::from("templates/basic-fungible-faucet.toml"),
        basic_fungible_faucet_library(),
        None,
    );

    build_package(
        &PathBuf::from("templates/basic-auth.toml"),
        rpo_falcon_512_library(),
        Some("auth"),
    );

    build_package(
        &PathBuf::from("templates/ecdsa-auth.toml"),
        ecdsa_k256_keccak_library(),
        Some("auth"),
    );

    build_package(&PathBuf::from("templates/no-auth.toml"), no_auth_library(), Some("auth"));

    build_package(
        &PathBuf::from("templates/multisig-auth.toml"),
        rpo_falcon_512_multisig_library(),
        Some("auth"),
    );

    build_package(
        &PathBuf::from("templates/acl-auth.toml"),
        rpo_falcon_512_acl_library(),
        Some("auth"),
    );
}

/// Builds a package and stores it under `{OUT_DIR}/{PACKAGE_DIR}` or
/// `{OUT_DIR}/{PACKAGE_DIR}/{subdirectory}` if a subdirectory is provided.
pub fn build_package(metadata_path: &Path, library: Library, subdirectory: Option<&str>) {
    let toml_string = fs::read_to_string(metadata_path)
        .unwrap_or_else(|_| panic!("Failed to read file {}", metadata_path.display()));

    let template_metadata =
        AccountComponentMetadata::from_toml(&toml_string).unwrap_or_else(|err| {
            panic!("Failed to deserialize component metadata in {}: {err}", metadata_path.display())
        });

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
        Section::new(SectionId::ACCOUNT_COMPONENT_METADATA, template_metadata.to_bytes());

    let package = Package {
        name: template_metadata.name().to_string(),
        version: Some(template_metadata.version().clone()),
        description: Some(template_metadata.description().to_string()),
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

    let mut output_filename = metadata_path
        .file_stem()
        .expect("metadata path should have a file stem")
        .to_os_string();
    output_filename.push(format!(".{MIDEN_PACKAGE_EXTENSION}"));

    let output_file = packages_out_dir.join(&output_filename);

    fs::write(&output_file, package.to_bytes()).unwrap_or_else(|e| {
        panic!(
            "Failed to write Package {} to file {} in {}. Error: {}",
            package.name,
            output_filename.display(),
            out_dir,
            e
        );
    });
}
