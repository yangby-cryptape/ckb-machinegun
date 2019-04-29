// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{thread, time};

pub(crate) fn sleep_millis(millis: u64) {
    let wait_millis = time::Duration::from_millis(millis);
    thread::sleep(wait_millis);
}

pub(crate) fn sleep_secs(secs: u64) {
    let wait_secs = time::Duration::from_secs(secs);
    thread::sleep(wait_secs);
}
