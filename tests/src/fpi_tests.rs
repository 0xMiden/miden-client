#[tokio::test]
async fn standard_fpi_public() {
    miden_client_integration_tests::tests::fpi::standard_fpi_public(Default::default()).await
}

#[tokio::test]
async fn standard_fpi_private() {
    miden_client_integration_tests::tests::fpi::standard_fpi_private(Default::default()).await
}

#[tokio::test]
async fn fpi_execute_program() {
    miden_client_integration_tests::tests::fpi::fpi_execute_program(Default::default()).await
}

#[tokio::test]
async fn nested_fpi_calls() {
    miden_client_integration_tests::tests::fpi::nested_fpi_calls(Default::default()).await
}
