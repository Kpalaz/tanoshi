#[macro_use]
extern crate log;
extern crate argon2;

pub mod application;
#[cfg(feature = "embed")]
pub mod assets;
pub mod auth;
pub mod config;
pub mod db;
pub mod domain;
pub mod graphql;
pub mod infrastructure;
pub mod notifier;
pub mod presentation;
pub mod proxy;
#[cfg(feature = "server")]
pub mod server;
pub mod utils;
pub mod worker;
