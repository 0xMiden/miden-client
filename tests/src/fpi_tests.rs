#[tokio::test]
async fn standard_fpi_public() {
    miden_client_integration_tests::tests::fpi::standard_fpi_public().await
}

#[tokio::test]
async fn standard_fpi_private() {
    miden_client_integration_tests::tests::fpi::standard_fpi_private().await
}

#[tokio::test]
async fn fpi_execute_program() {
    miden_client_integration_tests::tests::fpi::fpi_execute_program().await
}

#[tokio::test]
async fn nested_fpi_calls() {
    miden_client_integration_tests::tests::fpi::nested_fpi_calls().await
}
