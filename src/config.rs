// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::PathBuf;

use property::Property;

pub(crate) enum AppConfig {
    SyncCmd(SyncArgs),
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
            ("sync", Some(matches)) => AppConfig::SyncCmd(SyncArgs::from(matches)),
            _ => unreachable!(),
        }
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
