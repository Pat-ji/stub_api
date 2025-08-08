#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(static_mut_refs)]
#![allow(unused_must_use)]

pub use chrono;
pub use eframe;
pub use egui_extras;
pub use log;
pub use rand;
pub use rand_distr;
pub use regex;
pub use reqwest;
pub use scraper;
pub use serde;
pub use serde_json;
pub use strum;
pub use strum_macros;
pub use windows;
pub use noise;

pub mod api;

pub mod c_vec;
pub(crate) mod c_address;
pub(crate) mod offsets;
pub const API_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "script")]
pub use script_macro::script_exports;
