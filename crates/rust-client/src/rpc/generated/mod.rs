#[cfg(feature = "std")]
#[rustfmt::skip]
#[allow(clippy::trivially_copy_pass_by_ref)]
#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]
#[allow(dead_code)]
mod std;
#[cfg(feature = "std")]
pub use self::std::*;

#[cfg(not(feature = "std"))]
#[rustfmt::skip]
#[allow(clippy::trivially_copy_pass_by_ref)]
#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]
#[allow(dead_code)]
mod nostd;
#[cfg(not(feature = "std"))]
pub use nostd::*;
