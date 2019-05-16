// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ckb_jsonrpc_interfaces::{bytes, core, H256};

pub const SAFE_NUMBER_DISTANCE: core::BlockNumber = 10;

pub fn calculate_safe_number(tip_number: core::BlockNumber) -> Option<core::BlockNumber> {
    if tip_number < SAFE_NUMBER_DISTANCE {
        None
    } else {
        Some(tip_number - SAFE_NUMBER_DISTANCE)
    }
}

pub fn build_input(
    tx_hash: H256,
    index: u32,
    since: u64,
    args: Vec<bytes::Bytes>,
) -> core::transaction::CellInput {
    let cell_out_point = core::transaction::CellOutPoint { tx_hash, index };
    let outpoint = core::transaction::OutPoint {
        cell: Some(cell_out_point),
        block_hash: None,
    };
    core::transaction::CellInput::new(outpoint, since, args)
}

pub fn build_output(
    shannons: u64,
    data: bytes::Bytes,
    lock: core::script::Script,
    type_opt: Option<core::script::Script>,
) -> core::transaction::CellOutput {
    let capacity = core::Capacity::shannons(shannons);
    core::transaction::CellOutput::new(capacity, data, lock, type_opt)
}
