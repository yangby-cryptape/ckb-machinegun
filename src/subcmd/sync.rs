// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use console::Term;
use indicatif::{ProgressBar, ProgressStyle};

use ckb_jsonrpc_client::{
    client::CkbSyncClient,
    storage::{Storage, StorageReader, StorageWriter},
};

use crate::{chain, config::SyncArgs};

pub(crate) fn execute(args: SyncArgs) {
    let path = args.get_path();
    let url = args.get_url();

    let storage = Storage::initial(path.to_str().unwrap()).expect("failed to open storage");
    let client = CkbSyncClient::new(url.as_str());

    let tip_number = client
        .tip_block_number()
        .expect("failed to fetch tip number");
    let safe_number = chain::calculate_safe_number(tip_number).expect("no safe number");
    let mut start_number = storage.select_max_number().unwrap().unwrap_or(0);

    if start_number < safe_number {
        Term::stdout()
            .clear_screen()
            .and_then(|_| Term::stderr().clear_screen())
            .expect("failed to clear screen");

        let pb = ProgressBar::new(safe_number);
        pb.set_position(start_number);
        let style = ProgressStyle::default_bar()
        .template(
            "[{elapsed_precise:.bold.dim}] {percent:>3}% {bar:64.cyan/blue} {pos:>8}/{len:8} {prefix:12.cyan.bold.dim} {msg}",
        )
        .progress_chars("=>-");
        pb.set_style(style);
        pb.set_prefix(" Sync");
        pb.set_message("downloading chain data ...");

        while start_number <= safe_number {
            let block = client
                .block_by_number(start_number)
                .expect("failed to fetch block");
            storage.insert_block(&block).unwrap();
            start_number += 1;
            pb.inc(1);
        }
    }
}
