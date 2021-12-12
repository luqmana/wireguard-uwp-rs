//! This crate contains the `IVpnPlugIn` implementation for our UWP VPN plugin app.

#![windows_subsystem = "windows"]
#![allow(non_snake_case)] // Windows naming conventions

mod background;
mod config;
mod logging;
mod plugin;
mod utils;
