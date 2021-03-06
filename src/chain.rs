// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ckb_jsonrpc_interfaces::core;

pub const SAFE_NUMBER_DISTANCE: core::BlockNumber = 10;

pub fn calculate_safe_number(tip_number: core::BlockNumber) -> Option<core::BlockNumber> {
    if tip_number < SAFE_NUMBER_DISTANCE {
        None
    } else {
        Some(tip_number - SAFE_NUMBER_DISTANCE)
    }
}
