#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use anyhow::Result;

pub mod data;


pub mod utils;

#[cxx::bridge(namespace = "moe::kuriko::rust")]
mod ffi {

    extern "Rust" {

        fn load_resource_dat() -> i32;

    }

}

/// Load resource dat from current folder.
pub fn load_resource_dat() -> i32 {

    0
}

