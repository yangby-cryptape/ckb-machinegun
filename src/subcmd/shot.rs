// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::convert::TryInto;

use ckb_jsonrpc_client::{client::CkbSyncClient, storage::Storage, system};
use ckb_jsonrpc_interfaces::{core, types};

use crate::{chain, config::ShotArgs, wallet::Wallet};

pub(crate) fn execute(args: ShotArgs) {
    let path = args.get_path();
    let url = args.get_url();
    let key_in = args.get_key_in();
    let key_out = args.get_key_out();

    let storage = Storage::initial(path).expect("failed to open storage");
    let client = CkbSyncClient::new(url.to_owned());

    let pubkey_in = key_in.pubkey().expect("failed to genrate public key");
    let arg_in = system::calculate_arg(&pubkey_in);
    let arg_in = types::JsonBytes::from_vec(Vec::from(&arg_in[..]));

    let pubkey_out = key_out.pubkey().expect("failed to genrate public key");
    let arg_out = system::calculate_arg(&pubkey_out);
    let arg_out = types::JsonBytes::from_vec(Vec::from(&arg_out[..]));

    let (code_hash, dep) = storage.secp256k1_code_hash_and_dep().unwrap().unwrap();

    let lock_in = types::Script {
        args: vec![arg_in],
        code_hash: code_hash.clone(),
    };
    let lock_out = types::Script {
        args: vec![arg_out],
        code_hash,
    };

    let (outpoint, output) = storage
        .take_a_cell(&lock_in)
        .unwrap()
        .expect("no unspent cell");
    let input = chain::build_input(outpoint.tx_hash, outpoint.index.0 as u32, 0, Vec::new());
    let output = chain::build_output(
        output.capacity.0.as_u64(),
        Default::default(),
        lock_out.try_into().unwrap(),
        None,
    );
    let tx = core::transaction::TransactionBuilder::default()
        .dep(dep)
        .input(input)
        .output(output)
        .build();

    let witness = system::calculate_witness(&key_in, tx.hash());
    let tx = core::transaction::TransactionBuilder::from_transaction(tx)
        .witness(witness)
        .build();
    let tx: types::Transaction = (&tx).into();

    let hash = client.send(tx).expect("send tx");

    let red = console::Style::new().red();
    println!("{}", red.apply_to(&format!("tx hash = {:#x}", hash)));
}
