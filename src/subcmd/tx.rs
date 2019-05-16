// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::config::TxArgs;

use ckb_jsonrpc_client::storage::{Storage, StorageReader};

pub(crate) fn execute(args: TxArgs) {
    let path = args.get_path();
    let hash = args.get_hash();

    let storage = Storage::initial(path).expect("failed to open storage");

    let tx = storage
        .select_transaction(hash)
        .unwrap()
        .expect("transaction is not existed");
    let output = serde_json::to_string_pretty(&tx).unwrap();
    println!("{}", output);
}
