fn main() {
    #[cfg(feature = "napi")]
    {
        extern crate napi_build;
        napi_build::setup();
    }
}
