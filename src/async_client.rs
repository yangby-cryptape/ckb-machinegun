// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;

use crossbeam::channel::Sender;
use futures::Future;
use parking_lot::Mutex;
use rand::{thread_rng, Rng};
use tokio::runtime::Runtime;

use ckb_jsonrpc_interfaces::{types, Ckb, H256};
use jsonrpc_sdk_client::r#async::Client;
use jsonrpc_sdk_prelude::Error;

use crate::config::Node;

pub(crate) struct CkbClient {
    cli: Arc<Client>,
    nodes: Vec<Node>,
    rt: Mutex<Runtime>,
}

impl CkbClient {
    pub(crate) fn new(nodes: &[Node]) -> Self {
        Self {
            cli: Arc::new(Client::new()),
            nodes: nodes.to_vec(),
            rt: Mutex::new(Runtime::new().unwrap()),
        }
    }

    fn url(&self) -> String {
        let mut rng = thread_rng();
        let num: usize = rng.gen();
        let idx = num % self.nodes.len();
        let url = format!("http://{}:{}", self.nodes[idx].host, self.nodes[idx].port);
        url
    }

    pub fn send(&self, tx: types::Transaction) -> impl Future<Item = H256, Error = Error> {
        let url = self.url();
        self.cli
            .post(&url)
            .send(Ckb::send_transaction(tx), Default::default())
            .map(::std::convert::Into::into)
    }

    pub(crate) fn send_txs(
        &self,
        data: Vec<(H256, u32, types::Transaction)>,
        storage: Arc<Sender<(H256, u32, H256)>>,
        screen: Arc<Sender<bool>>,
    ) {
        for (h, i, t) in data.into_iter() {
            let storage = Arc::clone(&storage);
            let screen = Arc::clone(&screen);
            let task = self
                .send(t)
                .map(move |tx_hash| {
                    storage.send((h, i, tx_hash)).unwrap();
                })
                .then(move |res| {
                    screen.send(res.is_ok()).unwrap();
                    Ok(())
                });
            {
                let wait_time = ::std::time::Duration::from_secs(60 * 60 * 24);
                let mut rt = self.rt.try_lock_for(wait_time).unwrap();
                rt.spawn(task);
            }
        }
    }
}
