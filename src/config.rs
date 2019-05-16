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
    Key(KeyArgs),
    Tx(TxArgs),
    Sync(SyncArgs),
    Shot(ShotArgs),
}

pub(crate) struct KeyArgs {
    pub(crate) secret: Option<secp256k1::Privkey>,
}

#[derive(Property)]
pub(crate) struct TxArgs {
    path: PathBuf,
    hash: H256,
}

#[derive(Property)]
pub(crate) struct SyncArgs {
    path: PathBuf,
    url: url::Url,
}

#[derive(Property)]
pub(crate) struct ShotArgs {
    path: PathBuf,
    url: url::Url,
    key_in: secp256k1::Privkey,
    key_out: secp256k1::Privkey,
}

pub(crate) fn build_commandline() -> AppConfig {
    let yaml = clap::load_yaml!("cli.yaml");
    let matches = clap::App::from_yaml(yaml).get_matches();
    AppConfig::from(&matches)
}

impl<'a> From<&'a clap::ArgMatches<'a>> for AppConfig {
    fn from(matches: &'a clap::ArgMatches) -> Self {
        match matches.subcommand() {
            ("key", Some(matches)) => AppConfig::Key(KeyArgs::from(matches)),
            ("tx", Some(matches)) => AppConfig::Tx(TxArgs::from(matches)),
            ("sync", Some(matches)) => AppConfig::Sync(SyncArgs::from(matches)),
            ("shot", Some(matches)) => AppConfig::Shot(ShotArgs::from(matches)),
            _ => unreachable!(),
        }
    }
}

impl<'a> From<&'a clap::ArgMatches<'a>> for KeyArgs {
    fn from(matches: &'a clap::ArgMatches) -> Self {
        let secret = matches
            .value_of("secret")
            .map(|secret| parse_h256(&secret).into());
        Self { secret }
    }
}

impl<'a> From<&'a clap::ArgMatches<'a>> for TxArgs {
    fn from(matches: &'a clap::ArgMatches) -> Self {
        let path = value_t!(matches, "path", PathBuf).unwrap_or_else(|e| e.exit());
        let hash = value_t!(matches, "hash", String)
            .map(|ref x| parse_h256(x))
            .unwrap_or_else(|e| e.exit());
        Self { path, hash }
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

impl<'a> From<&'a clap::ArgMatches<'a>> for ShotArgs {
    fn from(matches: &'a clap::ArgMatches) -> Self {
        let path = value_t!(matches, "path", PathBuf).unwrap_or_else(|e| e.exit());
        let url_string = value_t!(matches, "url", String).unwrap_or_else(|e| e.exit());
        let url = url::Url::parse(&url_string).expect("please provide a valid url");
        let key_in = value_t!(matches, "key-in", String)
            .map(|ref x| parse_h256(x))
            .unwrap_or_else(|e| e.exit());
        let key_out = value_t!(matches, "key-out", String)
            .map(|ref x| parse_h256(x))
            .unwrap_or_else(|e| e.exit());
        Self {
            path,
            url,
            key_in: key_in.into(),
            key_out: key_out.into(),
        }
    }
}

fn parse_h256(h256_str: &str) -> H256 {
    if h256_str.len() != 64 + 2 || &h256_str[0..2] != "0x" {
        panic!("the format of input is not right");
    }
    H256::from_hex_str(&h256_str[2..]).expect("the format of input is not right")
}
