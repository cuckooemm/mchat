/// The global confi file name
pub const CONFIG_FILENAME: &str = "Config.toml";

extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod config;
pub mod internal;
pub mod handler;