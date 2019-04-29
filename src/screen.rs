// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{io, sync};

use console::Term;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

pub(crate) struct Screen {
    multi_progress: MultiProgress,
    tip_block_number: sync::Arc<ProgressBar>,
    unspent_count: sync::Arc<ProgressBar>,
    sent_count: sync::Arc<ProgressBar>,
    stats_turn: sync::Arc<ProgressBar>,
    stats_last50: sync::Arc<ProgressBar>,
    historical: sync::Arc<ProgressBar>,
    sync_status: sync::Arc<ProgressBar>,
    stolen_status: sync::Arc<ProgressBar>,
    sent_status: sync::Arc<ProgressBar>,
    check_status: sync::Arc<ProgressBar>,
}

macro_rules! progress_bar {
        ($name:ident) => {
            pub(crate) fn $name(&self) -> sync::Arc<ProgressBar> {
                sync::Arc::clone(&self.$name)
            }
        }
    }

impl Screen {
    pub(crate) fn clear() -> io::Result<()> {
        Term::stdout()
            .clear_screen()
            .and_then(|_| Term::stderr().clear_screen())
    }

    pub(crate) fn join_and_clear(&self) -> io::Result<()> {
        self.multi_progress.join_and_clear()
    }

    progress_bar!(tip_block_number);
    progress_bar!(unspent_count);
    progress_bar!(sent_count);
    progress_bar!(stats_turn);
    progress_bar!(stats_last50);
    progress_bar!(historical);
    progress_bar!(sync_status);
    progress_bar!(stolen_status);
    progress_bar!(sent_status);
    progress_bar!(check_status);

    pub(crate) fn new() -> Self {
        let mp = MultiProgress::new();
        mp.set_draw_target(ProgressDrawTarget::stdout());

        let msg_style = ProgressStyle::default_bar()
            .template("            {spinner:>2}  {prefix:10.cyan.bold.dim} {wide_msg}")
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let bar_style = ProgressStyle::default_bar()
        .template(
            "[{elapsed_precise:.bold.dim}] {percent:>3}% {bar:64.cyan/blue} {pos:>8}/{len:8} {prefix:12.cyan.bold.dim} {msg}",
        )
        .progress_chars("=>-");

        let tip_block_number = {
            let pb = mp.add(ProgressBar::new(1));
            pb.set_style(msg_style.clone());
            pb.set_prefix(" TipNumber");
            pb.set_message("Launch ...");
            sync::Arc::new(pb)
        };

        let unspent_count = {
            let pb = mp.add(ProgressBar::new(1));
            pb.set_style(msg_style.clone());
            pb.set_prefix("   Cells  ");
            pb.set_message("Waiting ...");
            sync::Arc::new(pb)
        };

        let sent_count = {
            let pb = mp.add(ProgressBar::new(1));
            pb.set_style(msg_style.clone());
            pb.set_prefix("  SendTxs ");
            pb.set_message("Waiting ...");
            sync::Arc::new(pb)
        };

        let stats_turn = {
            let pb = mp.add(ProgressBar::new(1));
            pb.set_style(msg_style.clone());
            pb.set_prefix("Statistics");
            pb.set_message("Waiting ...");
            sync::Arc::new(pb)
        };

        let stats_last50 = {
            let pb = mp.add(ProgressBar::new(1));
            pb.set_style(msg_style.clone());
            pb.set_prefix("Statistics");
            pb.set_message("Waiting ...");
            sync::Arc::new(pb)
        };

        let historical = {
            let pb = mp.add(ProgressBar::new(1));
            pb.set_style(msg_style.clone());
            pb.set_prefix("Historical");
            pb.set_message("Waiting ...");
            sync::Arc::new(pb)
        };

        let sync_status = {
            let pb = mp.add(ProgressBar::new(1));
            pb.set_style(bar_style.clone());
            pb.set_prefix("Synced Block");
            pb.set_message("Waiting ...");
            sync::Arc::new(pb)
        };

        let stolen_status = {
            let pb = mp.add(ProgressBar::new(1));
            pb.set_style(bar_style.clone());
            pb.set_prefix("Stolen Block");
            pb.set_message("Waiting ...");
            sync::Arc::new(pb)
        };

        let sent_status = {
            let pb = mp.add(ProgressBar::new(1));
            pb.set_style(bar_style.clone());
            pb.set_prefix(" Send   Txs ");
            pb.set_message("Waiting ...");
            sync::Arc::new(pb)
        };

        let check_status = {
            let pb = mp.add(ProgressBar::new(1));
            pb.set_style(bar_style.clone());
            pb.set_prefix(" Check  Txs ");
            pb.set_message("Waiting ...");
            sync::Arc::new(pb)
        };

        Self {
            multi_progress: mp,
            tip_block_number,
            unspent_count,
            sent_count,
            stats_turn,
            stats_last50,
            historical,
            sync_status,
            stolen_status,
            sent_status,
            check_status,
        }
    }
}
