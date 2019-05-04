// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate clap;

mod async_client;
mod config;
mod screen;
mod storage;
mod sync_client;
mod utils;

use std::{sync::Arc, thread};

use ckb_jsonrpc_client::sync::CkbClient as CkbSyncClient;
use ckb_jsonrpc_interfaces::{bytes, core, h256, types, H256};

use async_client::CkbClient as CkbAsyncMultiRemotesClient;
use config::{build_commandline, parse_arguments};
use screen::Screen;
use storage::Storage;
use sync_client::CkbClientPlus as _;

const SAFE_NUMBER_INTERVAL: u64 = 10;
const MOST_UNSPENT_CELLS_COUNT: u64 = 50000;

fn main() {
    let matches = build_commandline().get_matches();
    let config = parse_arguments(matches);
    let url = format!("http://{}:{}", config.nodes[0].host, config.nodes[0].port);
    let path = format!("{}.db", config.id);
    let interval = config.interval;

    let succ_lock = core::script::Script::new(Vec::new(), h256!("0x1"));
    let my_lock = core::script::Script::new(
        vec![bytes::Bytes::from(&config.id.into_bytes()[..])],
        h256!("0x1"),
    );

    Screen::clear().unwrap();
    let sync_client = Arc::new(CkbSyncClient::new(&url));
    let async_client = Arc::new(CkbAsyncMultiRemotesClient::new(&config.nodes[1..]));
    let storage = Storage::open(&path).expect("failed to open database");
    storage.init().unwrap();
    let screen = Screen::new();

    if config.skip_before != 0 {
        storage.update_chain_status(config.skip_before, 0).unwrap();
    }
    if config.steal_since != 0 {
        storage.update_stolen_status(config.steal_since).unwrap();
    }

    refresh_status(Arc::clone(&sync_client), &screen, Arc::clone(&storage));

    steal_capacity(
        Arc::clone(&sync_client),
        &screen,
        Arc::clone(&storage),
        succ_lock.clone(),
        my_lock.clone(),
    );

    check_cells(&screen, Arc::clone(&storage));

    send_transactions(
        Arc::clone(&async_client),
        &screen,
        Arc::clone(&storage),
        my_lock.clone(),
        interval,
    );

    check_transactions(&screen, Arc::clone(&storage));

    let _ = screen.join_and_clear();
}

fn refresh_status(sync_client: Arc<CkbSyncClient>, screen: &Screen, storage: Arc<Storage>) {
    let tip_block_number = screen.tip_block_number();
    let sync_status = screen.sync_status();
    {
        let number = sync_client.tip_block_number().unwrap();
        storage.update_turn_status(number).unwrap();
    }
    let _ = thread::spawn(move || {
        let mut current_number = 0;
        loop {
            if let Ok(number) = sync_client.tip_block_number() {
                tip_block_number.set_message(&format!("Current: #{}", number));
                tip_block_number.tick();
                if current_number != number {
                    current_number = number;
                    storage.update_tip_status(number).unwrap();
                }
            }
            if current_number <= SAFE_NUMBER_INTERVAL {
                utils::sleep_secs(10);
                continue;
            }
            let safe_number = current_number - SAFE_NUMBER_INTERVAL;
            sync_status.set_length(safe_number);
            let mut chain_number_next = storage
                .select_chain_number()
                .unwrap()
                .map(|x| x + 1)
                .unwrap_or(0);
            if safe_number > chain_number_next {
                if chain_number_next > 0 {
                    sync_status.set_position(chain_number_next - 1);
                    sync_status.set_message(&format!("Syncing #{} ...", chain_number_next));
                    sync_status.tick();
                }
                let mut cnt = 5;
                loop {
                    if cnt == 0 || chain_number_next >= safe_number {
                        break;
                    }
                    cnt -= 1;
                    let block = sync_client
                        .block_by_number(Some(chain_number_next))
                        .unwrap();
                    let timestamp = block.header.inner.timestamp.parse::<u64>().unwrap();
                    let tx_cells = block
                        .transactions
                        .iter()
                        .map(|tx| {
                            tx.inner
                                .outputs
                                .iter()
                                .enumerate()
                                .map(|(index, output)| {
                                    (
                                        tx.hash.clone(),
                                        index as u32,
                                        output.capacity.parse::<u64>().unwrap(),
                                    )
                                })
                                .fold(Vec::new(), |mut all, one| {
                                    all.push(one);
                                    all
                                })
                        })
                        .fold(Vec::new(), |mut all, mut part| {
                            all.append(&mut part);
                            all
                        });
                    let tx_hashes = block
                        .transactions
                        .iter()
                        .map(|tx| tx.hash.clone())
                        .collect::<Vec<H256>>();
                    storage
                        .save_chain_transactions(chain_number_next, &tx_hashes[..])
                        .unwrap();
                    storage.save_chain_cells(&tx_cells[..]).unwrap();
                    storage
                        .update_chain_status(chain_number_next, timestamp)
                        .unwrap();
                    sync_status.inc(1);
                    sync_status.set_message(&format!(
                        "Synced at #{} ({} txs)",
                        chain_number_next,
                        tx_hashes.len()
                    ));
                    chain_number_next += 1;
                }
            }
            for _ in 0..64 {
                tip_block_number.tick();
                utils::sleep_millis(64);
            }
        }
    });
}

fn steal_capacity(
    sync_client: Arc<CkbSyncClient>,
    screen: &Screen,
    storage: Arc<Storage>,
    lock_in: core::script::Script,
    lock_out: core::script::Script,
) {
    let stolen_status = screen.stolen_status();
    let _ = thread::spawn(move || loop {
        let tip_number = storage.select_tip_status().unwrap();
        if tip_number <= SAFE_NUMBER_INTERVAL {
            utils::sleep_secs(10);
            continue;
        }
        let safe_number = tip_number - SAFE_NUMBER_INTERVAL;
        stolen_status.set_length(safe_number);
        let mut stolen_number = storage.select_stolen_status().unwrap();
        stolen_status.set_position(stolen_number);
        if stolen_number >= safe_number {
            utils::sleep_secs(10);
            stolen_status.set_message("Blocking ...");
            continue;
        }
        stolen_status.set_message("Checking unspent cells ...");
        loop {
            let unspent_cells_count = storage.count_unspent_cells().unwrap();
            stolen_status.tick();
            if unspent_cells_count > MOST_UNSPENT_CELLS_COUNT {
                stolen_status.set_message("Pausing (rich enough) ...");
                utils::sleep_secs(10);
                continue;
            }
            if stolen_number >= safe_number {
                break;
            }
            stolen_status.tick();
            stolen_status.set_message(&format!("Stealing cells from #{}", stolen_number));
            let tx_opt_res = sync_client.steal(
                &lock_in,
                &lock_out,
                Some(stolen_number),
                Some(stolen_number),
                !0,
            );
            if let Ok(tx_opt) = tx_opt_res {
                if let Some(tx) = tx_opt {
                    if let Ok(tx_hash) = sync_client.send(tx) {
                        storage
                            .save_stolen_transaction(stolen_number, tx_hash.clone())
                            .unwrap();
                        storage.update_stolen_status(stolen_number).unwrap();
                        stolen_number += 1;
                        stolen_status.inc(1);
                        stolen_status
                            .set_message(&format!("Stole cells from #{} (done)", stolen_number));
                    } else {
                        stolen_status.set_message(&format!(
                            "Failed to steal cells from #{} (sending)",
                            stolen_number
                        ));
                    }
                } else {
                    storage.update_stolen_status(stolen_number).unwrap();
                    stolen_number += 1;
                    stolen_status.inc(1);
                    stolen_status.set_message(&format!(
                        "Skip stealing cells from #{} (poor)",
                        stolen_number
                    ));
                    continue;
                }
            } else {
                stolen_status.set_message(&format!(
                    "Failed to steal cells from #{} (preparing)",
                    stolen_number
                ));
            }
            utils::sleep_secs(10);
        }
    });
}

fn check_cells(screen: &Screen, storage: Arc<Storage>) {
    let unspent_count = screen.unspent_count();
    let _ = thread::spawn(move || loop {
        let count = storage.count_unspent_cells().unwrap();
        unspent_count.set_message(&format!("Unspent: {} cells", count));
        unspent_count.tick();
        utils::sleep_secs(5);
    });
}

fn send_transactions(
    async_client: Arc<CkbAsyncMultiRemotesClient>,
    screen: &Screen,
    storage: Arc<Storage>,
    lock: core::script::Script,
    interval: u64,
) {
    let storage_sender = {
        let storage_async = Arc::clone(&storage);
        let (sender, receiver) = ::crossbeam::channel::unbounded();
        let _ = thread::spawn(move || loop {
            if let Ok((h, i, tx_hash)) = receiver.recv() {
                storage_async.save_transaction((h, i, tx_hash)).unwrap();
            } else {
                utils::sleep_secs(1);
            }
        });
        Arc::new(sender)
    };
    let screen_sender = {
        let sent_count = screen.sent_count();
        let sent_status = screen.sent_status();
        let (sender, receiver) = ::crossbeam::channel::unbounded();
        let _ = thread::spawn(move || {
            let mut sent_cnt = 0u64;
            let mut passed_cnt = 0u64;
            let mut failed_cnt = 0u64;
            loop {
                sent_count.set_message(&format!(
                    " Turn  : {} sent, {} passed, {} failed",
                    sent_cnt, passed_cnt, failed_cnt
                ));
                if let Ok(passed) = receiver.recv() {
                    sent_cnt += 1;
                    if passed {
                        passed_cnt += 1;
                    } else {
                        failed_cnt += 1;
                    }
                    sent_status.inc(1);
                } else {
                    utils::sleep_secs(1);
                }
            }
        });
        Arc::new(sender)
    };
    let sent_status = screen.sent_status();
    let _ = thread::spawn(move || {
        let mut fetch_cnt = 0u64;
        sent_status.set_message("Sending txs ...");
        loop {
            let batch_id = storage.create_unspent_cells_batch().unwrap();
            let cells = storage.fetch_unspent_cells(batch_id).unwrap();
            let cells_cnt = cells.len() as u64;
            if cells_cnt == 0 {
                utils::sleep_secs(5);
                continue;
            }
            fetch_cnt += cells_cnt;
            sent_status.set_length(fetch_cnt);
            let data = {
                let mut data = Vec::new();
                for (input, cap) in cells.into_iter() {
                    let output = core::transaction::CellOutput::new(
                        core::Capacity::shannons(cap),
                        bytes::Bytes::new(),
                        lock.clone(),
                        None,
                    );
                    let h = input.previous_output.tx_hash.clone();
                    let i = input.previous_output.index;
                    let tx = types::Transaction {
                        version: 0,
                        deps: vec![],
                        inputs: vec![input.into()],
                        outputs: vec![output.into()],
                        witnesses: vec![],
                    };
                    data.push((h, i, tx));
                }
                data
            };
            async_client.send_txs(
                data,
                Arc::clone(&storage_sender),
                Arc::clone(&screen_sender),
            );
            utils::sleep_millis(interval);
        }
    });
}

fn check_transactions(screen: &Screen, storage: Arc<Storage>) {
    let stats_turn = screen.stats_turn();
    let stats_last50 = screen.stats_last50();
    let historical = screen.historical();
    let check_status = screen.check_status();
    let number_keep = 50;
    let _ = thread::spawn(move || {
        check_status.set_message("Historical: Committed / Passed");
        loop {
            {
                let (sent_cnt, passed_cnt, committed_cnt) = storage.count_transactions().unwrap();
                historical.tick();
                historical.set_message(&format!(
                    " Total : {} sent, {} passed, {} committed",
                    sent_cnt, passed_cnt, committed_cnt
                ));
                if passed_cnt > 0 {
                    check_status.set_length(passed_cnt);
                    check_status.set_position(committed_cnt);
                }
            }
            let turn_number = storage.select_turn_status().unwrap();
            let (txs_cnt, start, end, min, max, avg) = storage.do_statistics(turn_number).unwrap();
            if txs_cnt != 0 && start != 0 && end != 0 {
                let cost_secs = (end - start) as f32 / 1000.0;
                let tps = txs_cnt as f32 / cost_secs;
                let min = min as f32 / 1000.0;
                let max = max as f32 / 1000.0;
                let avg = (avg as f32) / (txs_cnt as f32) / 1000.0;
                stats_turn.tick();
                stats_turn.set_message(&format!(
                    " Turn  : {} txs, cost {:.2}s, {:.2} tps, min {:.2}s, max {:.2}s, avg {:.2}s",
                    txs_cnt, cost_secs, tps, min, max, avg
                ));
            }

            if let Some(chain_number) = storage.select_chain_number().unwrap() {
                if chain_number > turn_number + number_keep {
                    let (txs_cnt, start, end, min, max, avg) =
                        storage.do_statistics(chain_number - number_keep).unwrap();
                    if txs_cnt != 0 && start != 0 && end != 0 {
                        let cost_secs = (end - start) as f32 / 1000.0;
                        let tps = txs_cnt as f32 / cost_secs;
                        let min = min as f32 / 1000.0;
                        let max = max as f32 / 1000.0;
                        let avg = (avg as f32) / (txs_cnt as f32) / 1000.0;
                        stats_last50.tick();
                        stats_last50.set_message(&format!(
                    "Last 50: {} txs, cost {:.2}s, {:.2} tps, min {:.2}s, max {:.2}s, avg {:.2}s",
                    txs_cnt, cost_secs, tps, min, max, avg
                ));
                    }
                }
            }
            utils::sleep_secs(5);
        }
    });
}
