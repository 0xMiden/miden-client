use miden_client::Felt as NativeFelt;
use napi::bindgen_prelude::*;

use super::napi_wrap;

napi_wrap!(copy Felt wraps NativeFelt);

#[napi]
impl Felt {
    #[napi(constructor)]
    pub fn new(value: BigInt) -> Result<Self> {
        let (_, value, _) = value.get_u64();
        Ok(Felt(NativeFelt::new(value)))
    }

    #[napi(js_name = "asInt")]
    pub fn as_int(&self) -> BigInt {
        BigInt::from(self.0.as_int())
    }

    #[napi(js_name = "toString")]
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}
