#[cfg(feature = "std")]
#[rustfmt::skip]
#[allow(dead_code)]
mod std_gen {
    include!(concat!(env!("OUT_DIR"), "/note_transport_std.rs"));
}
#[cfg(feature = "std")]
pub use std_gen::*;

#[cfg(not(feature = "std"))]
#[rustfmt::skip]
#[allow(dead_code)]
mod nostd_gen {
    include!(concat!(env!("OUT_DIR"), "/note_transport_nostd.rs"));
}
#[cfg(not(feature = "std"))]
pub use nostd_gen::*;
