#[macro_use]
extern crate thiserror;

mod client;
pub mod proto;

pub use crate::client::{Client, ClientError, Config};
