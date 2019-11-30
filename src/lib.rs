#[macro_use]
extern crate thiserror;

mod client;
pub mod proto;

use std::convert::TryInto;
use std::net::ToSocketAddrs;

use rustyline::{error::ReadlineError, Editor};
use tokio::prelude::*;

pub use crate::client::{Client, ClientError, Config};
