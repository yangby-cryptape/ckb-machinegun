// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::PathBuf;

use property::Property;

use ckb_jsonrpc_interfaces::{secp256k1, H256};

pub(crate) enum AppConfig {
    KeyCmd(KeyArgs),
    SyncCmd(SyncArgs),
}

pub(crate) struct KeyArgs {
    pub(crate) secret: Option<secp256k1::Privkey>,
}

#[derive(Property)]
pub(crate) struct SyncArgs {
    path: PathBuf,
    url: url::Url,
}

pub(crate) fn build_commandline() -> AppConfig {
    let yaml = clap::load_yaml!("cli.yaml");
    let matches = clap::App::from_yaml(yaml).get_matches();
    AppConfig::from(&matches)
}

impl<'a> From<&'a clap::ArgMatches<'a>> for AppConfig {
    fn from(matches: &'a clap::ArgMatches) -> Self {
        match matches.subcommand() {
            ("key", Some(matches)) => AppConfig::KeyCmd(KeyArgs::from(matches)),
            ("sync", Some(matches)) => AppConfig::SyncCmd(SyncArgs::from(matches)),
            _ => unreachable!(),
        }
    }
}

impl<'a> From<&'a clap::ArgMatches<'a>> for KeyArgs {
    fn from(matches: &'a clap::ArgMatches) -> Self {
        let secret = matches.value_of("secret").map(|secret| {
            if secret.len() != 64 + 2 || &secret[0..2] != "0x" {
                panic!("the format of input key is not right");
            }
            let secret_hash =
                H256::from_hex_str(&secret[2..]).expect("the format of input key is not right");
            secret_hash.into()
        });
        Self { secret }
    }
}

impl<'a> From<&'a clap::ArgMatches<'a>> for SyncArgs {
    fn from(matches: &'a clap::ArgMatches) -> Self {
        let path = value_t!(matches, "path", PathBuf).unwrap_or_else(|e| e.exit());
        let url_string = value_t!(matches, "url", String).unwrap_or_else(|e| e.exit());
        let url = url::Url::parse(&url_string).expect("please provide a valid url");
        Self { path, url }
    }
}
