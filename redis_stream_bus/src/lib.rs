pub mod bus;
pub mod client;
pub mod config;
pub mod error;

#[cfg(test)]
mod client_test;


//TODO Use Server to spin up new server for tests
// #[cfg(test)]
// mod server;


pub use internals::*;





// Re-export #[derive(Serialize, RedisStream)].
//
#[cfg(feature = "bus_derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate bus_derive;
#[cfg(feature = "bus_derive")]
#[doc(hidden)]
pub use bus_derive::*;