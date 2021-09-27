pub mod bus;
pub mod client;
pub mod config;
pub mod error;

#[cfg(test)]
mod client_test;


//TODO Use Server to spin up new server for tests
// #[cfg(test)]
// mod server;


pub use internals::StreamParsable;





// Re-export #[derive(Serialize, Deserialize)].
//
// The reason re-exporting is not enabled by default is that disabling it would
// be annoying for crates that provide handwritten impls or data formats. They
// would need to disable default features and then explicitly re-enable std.
#[cfg(feature = "bus_derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate bus_derive;
#[cfg(feature = "bus_derive")]
#[doc(hidden)]
pub use bus_derive::*;