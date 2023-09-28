#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use anyhow::Result;

pub mod data;


pub mod utils;

pub extern "C" fn load_config() -> i32 {

    0
}
