use std::error::Error;
use std::fmt::Write;

use miden_client::{ClientError, ErrorHint};

// ERROR HANDLING
// ================================================================================================

/// Converts a Rust error into a napi error with full source chain context.
pub(crate) fn to_napi_err<T: Error + 'static>(err: T, context: &str) -> napi::Error {
    let mut error_string = context.to_string();
    let mut source = Some(&err as &dyn Error);
    while let Some(e) = source {
        write!(error_string, ": {e}").expect("writing to string should always succeed");
        source = e.source();
    }

    if let Some(help) = hint_from_error(&err) {
        write!(error_string, "\nHelp: {help}").expect("writing to string should always succeed");
    }

    napi::Error::from_reason(error_string)
}

fn hint_from_error(err: &(dyn Error + 'static)) -> Option<String> {
    if let Some(client_error) = err.downcast_ref::<ClientError>() {
        return Option::<ErrorHint>::from(client_error).map(ErrorHint::into_help_message);
    }
    err.source().and_then(hint_from_error)
}
