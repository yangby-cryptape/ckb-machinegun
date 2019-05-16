// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate clap;

pub(crate) mod chain;
pub(crate) mod config;
pub(crate) mod subcmd;
pub(crate) mod wallet;

fn main() {
    let config = config::build_commandline();
    match config {
        config::AppConfig::Key(args) => subcmd::key::execute(args),
        config::AppConfig::Tx(args) => subcmd::tx::execute(args),
        config::AppConfig::Sync(args) => subcmd::sync::execute(args),
        config::AppConfig::Shot(args) => subcmd::shot::execute(args),
    }
}
