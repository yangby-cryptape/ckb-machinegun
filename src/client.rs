// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::convert::TryInto;

use ckb_jsonrpc_client::sync::CkbClient;
use ckb_jsonrpc_interfaces::{bytes, core, types, OccupiedCapacity};
use jsonrpc_sdk_prelude::{Error, Result};

pub(crate) trait CkbClientPlus {
    fn steal(
        &self,
        lock_in: &core::script::Script,
        lock_out: &core::script::Script,
        from: Option<core::BlockNumber>,
        to: Option<core::BlockNumber>,
        max_count: usize,
    ) -> Result<Option<types::Transaction>>;
}

impl CkbClientPlus for CkbClient {
    fn steal(
        &self,
        lock_in: &core::script::Script,
        lock_out: &core::script::Script,
        from: Option<core::BlockNumber>,
        to: Option<core::BlockNumber>,
        max_count: usize,
    ) -> Result<Option<types::Transaction>> {
        self.cells_by_lock_hash(lock_in, from, to).and_then(
            move |cells: Vec<types::CellOutputWithOutPoint>| {
                let total_capacity = cells
                    .iter()
                    .map(|c| c.capacity.parse::<u64>())
                    .collect::<::std::result::Result<Vec<_>, std::num::ParseIntError>>()
                    .map_err(|_| Error::custom("parse capacity failed"))
                    .and_then(|caps| {
                        caps.into_iter()
                            .try_fold(0u64, u64::checked_add)
                            .ok_or_else(|| Error::custom("sum capacity overflow"))
                    })?;
                let inputs = cells
                    .into_iter()
                    .map(|c| {
                        core::transaction::CellInput {
                            previous_output: c.out_point.try_into().unwrap(),
                            args: vec![],
                            since: 0,
                        }
                        .into()
                    })
                    .collect();
                let (output, least_capacity) = {
                    let mut output = core::transaction::CellOutput::new(
                        core::Capacity::shannons(0),
                        bytes::Bytes::new(),
                        lock_out.clone(),
                        None,
                    );
                    let least_capacity = output
                        .occupied_capacity()
                        .map_err(|_| Error::custom("least capacity capacity overflow"))?
                        .as_u64();
                    output.capacity = core::Capacity::shannons(least_capacity);
                    (output, least_capacity)
                };
                let tx_opt = if total_capacity > least_capacity {
                    let outputs = {
                        let mut output_size = (total_capacity / least_capacity) as usize - 1;
                        let mut remain_output_size = 0;
                        if output_size > max_count {
                            remain_output_size = output_size - max_count;
                            output_size = max_count;
                        }
                        let remain_capacity = total_capacity % least_capacity
                            + least_capacity * (remain_output_size as u64)
                            + least_capacity;

                        let mut outputs = vec![output; output_size];
                        let output = core::transaction::CellOutput::new(
                            core::Capacity::shannons(remain_capacity),
                            bytes::Bytes::new(),
                            lock_in.clone(),
                            None,
                        );
                        outputs.insert(0, output);
                        outputs
                    };
                    Some(types::Transaction {
                        version: 0,
                        deps: vec![],
                        inputs,
                        outputs: outputs.into_iter().map(Into::into).collect(),
                        witnesses: vec![],
                    })
                } else {
                    None
                };
                Ok(tx_opt)
            },
        )
    }
}
