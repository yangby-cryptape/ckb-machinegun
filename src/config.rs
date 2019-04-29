// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{fmt, str};

use clap::{App, Arg, ArgMatches};

const APPNAME: &str = "CKB MachineGun";
const VERNUM: &str = "0.1.0";

#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub(crate) host: String,
    pub(crate) port: u16,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

impl str::FromStr for Node {
    type Err = ParseNodeError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.split(':').collect::<Vec<&str>>();
        if v.len() != 2 {
            return Err(ParseNodeError {
                address: s.to_string(),
            });
        }
        let h = v[0].trim();
        let p = v[1].parse::<u16>();
        if h.is_empty() || p.is_err() {
            return Err(ParseNodeError {
                address: s.to_string(),
            });
        }
        Ok(Node {
            host: h.to_string(),
            port: p.unwrap(),
        })
    }
}

pub(crate) struct ParseNodeError {
    address: String,
}

impl fmt::Display for ParseNodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&format!("the address [{}] for node is malformed", &self.address)).fmt(f)
    }
}

pub(crate) struct AppConfig {
    pub(crate) node: Vec<Node>,
    pub(crate) id: String,
    pub(crate) interval: u64,
}

impl<'a> From<&'a ArgMatches<'a>> for AppConfig {
    fn from(matches: &'a ArgMatches) -> Self {
        let node = values_t!(matches, "node", Node).unwrap_or_else(|e| e.exit());
        let id = value_t!(matches, "id", String).unwrap_or_else(|e| e.exit());
        let interval = value_t!(matches, "interval", u64).unwrap_or_else(|e| e.exit());
        Self { node, id, interval }
    }
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = "\nAppConfig: {{\n".to_string();
        ret.push_str(&format!("    node[{}]:\n", self.node.len()));
        for node in self.node.iter() {
            ret.push_str(&format!("        {}\n", node));
        }
        ret.push_str(&format!("    id: {}\n", self.id));
        ret.push_str(&format!("    interval: {}\n", self.interval));
        ret += "}}\n";
        write!(f, "{}", ret)
    }
}

pub(crate) fn build_commandline<'a>() -> App<'a, 'a> {
    App::new(APPNAME)
        .version(VERNUM)
        .author("Boyu Yang <yangby@cryptape.com>")
        .about("A Machine Gun for attacking CKB through JSON-RPC.")
        .arg(
            Arg::with_name("node")
                .long("node")
                .short("N")
                .required(true)
                .takes_value(true)
                // TODO multi nodes
                //  .multiple(true)
                //  .value_delimiter(",")
                //  .help("Set the host:port[,host:port[...]] of nodes to send transactions."),
                .help("Set the host:port of nodes to send transactions."),
        )
        .arg(
            Arg::with_name("id")
                .long("id")
                .short("i")
                .required(true)
                .takes_value(true)
                .help(
                    "An unique ID to generate hash for your cells. \
                     It is also used as a database name.",
                ),
        )
        .arg(
            Arg::with_name("interval")
                .long("interval")
                .short("i")
                .takes_value(true)
                .default_value("500")
                .help("Wait interval millisecond between sending each request. 0 means no wait."),
        )
}

pub(crate) fn parse_arguments(matches: ArgMatches) -> AppConfig {
    AppConfig::from(&matches)
}
