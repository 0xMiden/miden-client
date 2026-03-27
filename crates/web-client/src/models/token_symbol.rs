use js_export_macro::js_export;
use miden_client::asset::TokenSymbol as NativeTokenSymbol;

use crate::js_error_with_context;
use crate::platform::JsErr;

/// Represents a string token symbol (e.g. "POL", "ETH") as a single {@link Felt | `Felt`} value.
///
/// Token Symbols can consists of up to 6 capital Latin characters, e.g. "C", "ETH", "MIDENC".
#[js_export]
#[derive(Clone)]
pub struct TokenSymbol(NativeTokenSymbol);

#[js_export]
impl TokenSymbol {
    /// Creates a token symbol from a string.
    #[js_export(constructor)]
    pub fn new(symbol: String) -> Result<TokenSymbol, JsErr> {
        let native_token_symbol = NativeTokenSymbol::new(&symbol)
            .map_err(|err| js_error_with_context(err, "failed to create token symbol"))?;
        Ok(TokenSymbol(native_token_symbol))
    }

    /// Returns the validated symbol string.
    #[js_export(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> Result<String, JsErr> {
        self.0
            .to_string()
            .map_err(|err| js_error_with_context(err, "failed to convert token symbol to string"))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeTokenSymbol> for TokenSymbol {
    fn from(native_token_symbol: NativeTokenSymbol) -> Self {
        TokenSymbol(native_token_symbol)
    }
}

impl From<&NativeTokenSymbol> for TokenSymbol {
    fn from(native_token_symbol: &NativeTokenSymbol) -> Self {
        TokenSymbol(*native_token_symbol)
    }
}

impl From<TokenSymbol> for NativeTokenSymbol {
    fn from(token_symbol: TokenSymbol) -> Self {
        token_symbol.0
    }
}

impl From<&TokenSymbol> for NativeTokenSymbol {
    fn from(token_symbol: &TokenSymbol) -> Self {
        token_symbol.0
    }
}
