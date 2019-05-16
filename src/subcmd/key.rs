// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::str::from_utf8_unchecked;

use ckb_jsonrpc_client::system;
use ckb_jsonrpc_interfaces::{secp256k1, H256};

use crate::config::KeyArgs;

pub(crate) fn execute(args: KeyArgs) {
    let cyan = console::Style::new().cyan();
    let privkey = args.secret.unwrap_or_else(|| {
        let privkey = secp256k1::Generator::new().random_privkey();
        let privkey_hash: H256 = privkey.clone();
        println!(
            "{}",
            cyan.apply_to(&format!("secret = {:#x}", privkey_hash))
        );
        privkey
    });
    let pubkey = privkey.pubkey().expect("failed to genrate public key");
    let arg = system::calculate_arg(&pubkey);
    let arg_string = {
        let mut buffer = vec![0u8; 20 * 2 + 2];
        buffer[0] = b'0';
        buffer[1] = b'x';
        faster_hex::hex_encode(&arg, &mut buffer[2..]).unwrap();
        unsafe { from_utf8_unchecked(&buffer) }.to_owned()
    };
    println!("{}", cyan.apply_to(&format!("  arg  = {}", arg_string)));
}
