use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use miden_client::account::component::AccountComponentMetadata;
use miden_client::assembly::{DefaultSourceManager, Library, LibraryPath, Module, ModuleKind};
use miden_client::transaction::TransactionKernel;
use miden_client::utils::Serializable;
use miden_client::vm::{
    MastArtifact, Package, PackageExport, PackageManifest, QualifiedProcedureName, Section,
    SectionId,
};
use tracing::info;

use crate::errors::CliError;

const MIDEN_PACKAGE_EXTENSION: &str = "masp";

#[derive(Debug, Parser, Clone)]
#[command(about = "Create a package from .masm file(s) and metadata")]
pub struct PackageCmd {
    /// Path to the metadata TOML file
    #[arg(short, long)]
    pub metadata: PathBuf,

    /// List of .masm source files to include in the package
    #[arg(short, long, required = true)]
    pub sources: Vec<PathBuf>,

    /// Output path for the generated .masp package file
    #[arg(short, long)]
    pub output: PathBuf,
}

impl PackageCmd {
    pub fn execute(&self) -> Result<(), CliError> {
        // Read and parse the metadata file
        let metadata_content = fs::read_to_string(&self.metadata).map_err(|err| {
            CliError::PackageError(
                Box::new(err),
                format!("Failed to read metadata file {}", self.metadata.display()),
            )
        })?;

        let component_metadata =
            AccountComponentMetadata::from_toml(&metadata_content).map_err(|err| {
                CliError::PackageError(
                    Box::new(err),
                    format!(
                        "Failed to deserialize component metadata from {}",
                        self.metadata.display()
                    ),
                )
            })?;

        info!("Creating package '{}'", component_metadata.name());

        // Read all source files and compile them into a library
        let library = self.compile_library(&component_metadata)?;

        // Build the package
        let package = self.build_package(component_metadata, library)?;

        // Write the package to the output file
        self.write_package(&package)?;

        println!(
            "Successfully created package '{}' at {}",
            package.name,
            self.output.display()
        );

        Ok(())
    }

    fn compile_library(
        &self,
        component_metadata: &AccountComponentMetadata,
    ) -> Result<Library, CliError> {
        let assembler = TransactionKernel::assembler();
        let source_manager = Arc::new(DefaultSourceManager::default());

        // Read all source files and parse them into modules
        let mut modules = Vec::new();
        for source_path in &self.sources {
            let source_code = fs::read_to_string(source_path).map_err(|err| {
                CliError::PackageError(
                    Box::new(err),
                    format!("Failed to read source file {}", source_path.display()),
                )
            })?;

            // Extract the module name from the filename (without extension)
            let module_name = source_path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| {
                    CliError::PackageError(
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Invalid file name",
                        )),
                        format!("Could not extract module name from {}", source_path.display()),
                    )
                })?;

            // Create a library path that includes the package name and module name
            let library_path_str = format!("{}::{}", component_metadata.name(), module_name);
            let library_path = LibraryPath::try_from(library_path_str.as_str())
                .map_err(|err| {
                    CliError::PackageError(
                        Box::new(err),
                        format!(
                            "Invalid library path '{}'",
                            library_path_str
                        ),
                    )
                })?;

            // Parse the module using Module::parser
            let module = Module::parser(ModuleKind::Library)
                .parse_str(library_path, source_code, &source_manager)
                .map_err(|err| {
                    CliError::PackageError(
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())),
                        format!("Failed to parse module from {}", source_path.display()),
                    )
                })?;

            modules.push(module);
        }

        // Compile all modules into a library
        let library = assembler
            .assemble_library(modules)
            .map_err(|err| {
                CliError::PackageError(
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())),
                    "Failed to assemble library from source files".to_string(),
                )
            })?;

        Ok(library)
    }

    fn build_package(
        &self,
        component_metadata: AccountComponentMetadata,
        library: Library,
    ) -> Result<Package, CliError> {
        // Gather all of the procedure metadata for exports of this package
        let mut exports: Vec<PackageExport> = Vec::new();
        for module_info in library.module_infos() {
            for (_, proc_info) in module_info.procedures() {
                let name =
                    QualifiedProcedureName::new(module_info.path().clone(), proc_info.name.clone());
                let digest = proc_info.digest;
                let signature = proc_info.signature.as_deref().cloned();
                let attributes = proc_info.attributes.clone();
                exports.push(PackageExport { name, digest, signature, attributes });
            }
        }

        let mast = MastArtifact::Library(Arc::new(library));
        let manifest = PackageManifest::new(exports);

        let account_component_metadata_section =
            Section::new(SectionId::ACCOUNT_COMPONENT_METADATA, component_metadata.to_bytes());

        let package = Package {
            name: component_metadata.name().to_string(),
            version: Some(component_metadata.version().clone()),
            description: Some(component_metadata.description().to_string()),
            mast,
            manifest,
            sections: vec![account_component_metadata_section],
        };

        Ok(package)
    }

    fn write_package(&self, package: &Package) -> Result<(), CliError> {
        // Ensure the output has the correct extension
        let output_path = if self.output.extension().and_then(|s| s.to_str()) != Some(MIDEN_PACKAGE_EXTENSION) {
            self.output.with_extension(MIDEN_PACKAGE_EXTENSION)
        } else {
            self.output.clone()
        };

        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                CliError::PackageError(
                    Box::new(err),
                    format!("Failed to create output directory {}", parent.display()),
                )
            })?;
        }

        // Write the package bytes
        fs::write(&output_path, package.to_bytes()).map_err(|err| {
            CliError::PackageError(
                Box::new(err),
                format!("Failed to write package to {}", output_path.display()),
            )
        })?;

        Ok(())
    }
}
