mod client;
mod proto;

use std::convert::TryInto;
use std::net::ToSocketAddrs;

use rustyline::{error::ReadlineError, Editor};
use tokio::prelude::*;

use crate::client::{Client, ClientError, Config};
