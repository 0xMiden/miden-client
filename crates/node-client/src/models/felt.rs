use miden_client::Felt as NativeFelt;
use napi::bindgen_prelude::*;

#[napi]
#[derive(Clone, Copy)]
pub struct Felt(pub(crate) NativeFelt);

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

impl From<NativeFelt> for Felt {
    fn from(native: NativeFelt) -> Self {
        Felt(native)
    }
}

impl From<&NativeFelt> for Felt {
    fn from(native: &NativeFelt) -> Self {
        Felt(*native)
    }
}

impl From<Felt> for NativeFelt {
    fn from(felt: Felt) -> Self {
        felt.0
    }
}

impl From<&Felt> for NativeFelt {
    fn from(felt: &Felt) -> Self {
        felt.0
    }
}
