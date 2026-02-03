use alloc::sync::Arc;

use miden_client::Serializable;
use miden_client::account::AccountId as NativeAccountId;
use miden_client::assembly::Assembler as NativeAssembler;
use miden_client::testing::account_id::ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE;
use miden_client::vm::{
    MastArtifact as NativeMastArtifact, Package as NativePackage, PackageKind as NativePackageKind,
    PackageManifest as NativePackageManifest,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::account_id::AccountId;

#[wasm_bindgen]
pub struct TestUtils;

#[wasm_bindgen]
impl TestUtils {
    #[wasm_bindgen(js_name = "createMockAccountId")]
    pub fn create_mock_account_id() -> AccountId {
        let native_account_id: NativeAccountId =
            ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE.try_into().unwrap();
        native_account_id.into()
    }

    #[wasm_bindgen(js_name = "createMockSerializedLibraryPackage")]
    pub fn create_mock_serialized_library_package() -> Uint8Array {
        pub const CODE: &str = "
            pub proc foo
                push.1.2 mul
            end

            pub proc bar
                push.1.2 add
            end
        ";

        let library = NativeAssembler::default().assemble_library([CODE]).unwrap();

        let package_without_metadata = NativePackage {
            name: "test_package_no_metadata".to_string(),
            mast: NativeMastArtifact::Library(Arc::new(library)),
            manifest: NativePackageManifest::new(None),
            sections: vec![], // No metadata section
            version: Default::default(),
            description: None,
            kind: NativePackageKind::Library,
        };

        let bytes: Vec<u8> = package_without_metadata.to_bytes();

        Uint8Array::from(bytes.as_slice())
    }

    #[wasm_bindgen(js_name = "createMockSerializedProgramPackage")]
    pub fn create_mock_serialized_program_package() -> Uint8Array {
        pub const CODE: &str = "
            begin
                # This code computes 1001st Fibonacci number
                repeat.1000
                    swap dup.1 add
                end
            end
        ";

        let program = NativeAssembler::default().assemble_program(CODE).unwrap();

        let package_without_metadata = NativePackage {
            name: "test_program_package_no_metadata".to_string(),
            mast: NativeMastArtifact::Executable(Arc::new(program)),
            manifest: NativePackageManifest::new(None),
            sections: vec![], // No metadata section
            version: Default::default(),
            description: None,
            kind: NativePackageKind::Executable,
        };

        let bytes: Vec<u8> = package_without_metadata.to_bytes();

        Uint8Array::from(bytes.as_slice())
    }
}
