fn main() {
    #[cfg(feature = "nodejs")]
    napi_build::setup();
}
