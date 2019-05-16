// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ckb_jsonrpc_client::{
    error::Result,
    storage::{StorageReader, StorageWriter},
    system,
};
use ckb_jsonrpc_interfaces::{core, types, H256};

const FLAG_DIRTY: u8 = 1;
const FLAG_CHECKED: u8 = 2;

pub trait WalletWriter {
    fn insert_number(&self, flag: u8, key: &[u8], number: u64) -> Result<()>;
    fn insert_dirty_number(&self, key: &[u8], number: u64) -> Result<()>;
    fn insert_checked_number(&self, key: &[u8], number: u64) -> Result<()>;
}

pub trait WalletReader {
    fn select_number(&self, flag: u8, key: &[u8]) -> Result<Option<u64>>;
    fn select_dirty_number(&self, key: &[u8]) -> Result<Option<u64>>;
    fn select_checked_number(&self, key: &[u8]) -> Result<Option<u64>>;
}

pub trait Wallet {
    fn secp256k1_code_hash_and_dep(&self) -> Result<Option<(H256, core::transaction::OutPoint)>>;
    fn take_a_cell(
        &self,
        lock_script: &types::Script,
    ) -> Result<Option<(types::CellOutPoint, types::CellOutput)>>;
}

impl<T> WalletWriter for T
where
    T: StorageWriter,
{
    fn insert_number(&self, flag: u8, key: &[u8], number: u64) -> Result<()> {
        let mut vec = Vec::with_capacity(key.len() + 1);
        vec.push(flag);
        vec.extend_from_slice(&key);
        self.insert(&vec, &number.to_le_bytes())
    }

    fn insert_dirty_number(&self, key: &[u8], number: u64) -> Result<()> {
        self.insert_number(FLAG_DIRTY, key, number)
    }

    fn insert_checked_number(&self, key: &[u8], number: u64) -> Result<()> {
        self.insert_number(FLAG_CHECKED, key, number)
    }
}

impl<T> WalletReader for T
where
    T: StorageReader,
{
    fn select_number(&self, flag: u8, key: &[u8]) -> Result<Option<u64>> {
        let mut vec = Vec::with_capacity(key.len() + 1);
        vec.push(flag);
        vec.extend_from_slice(&key);
        self.select(&vec).map(|opt| {
            opt.map(|vector| {
                let mut bytes = [0u8; 8];
                (&mut bytes).copy_from_slice(&vector);
                u64::from_le_bytes(bytes)
            })
        })
    }

    fn select_dirty_number(&self, key: &[u8]) -> Result<Option<u64>> {
        self.select_number(FLAG_DIRTY, key)
    }

    fn select_checked_number(&self, key: &[u8]) -> Result<Option<u64>> {
        self.select_number(FLAG_CHECKED, key)
    }
}

impl<T> Wallet for T
where
    T: StorageWriter + StorageReader + WalletWriter + WalletReader,
{
    fn secp256k1_code_hash_and_dep(&self) -> Result<Option<(H256, core::transaction::OutPoint)>> {
        let genesis_block_opt = self.select_block_by_number(0)?;
        if genesis_block_opt.is_none() {
            return Ok(None);
        }
        let genesis_block = genesis_block_opt.unwrap();
        Ok(Some(system::calculate_secp256k1_code_hash_and_dep(
            &genesis_block,
        )))
    }

    fn take_a_cell(
        &self,
        lock_script: &types::Script,
    ) -> Result<Option<(types::CellOutPoint, types::CellOutput)>> {
        let max_number_opt = self.select_max_number()?;
        if max_number_opt.is_none() {
            return Ok(None);
        }
        let lock_hash = {
            use std::convert::TryInto;
            let lock_script: core::script::Script = lock_script.clone().try_into().unwrap();
            lock_script.hash()
        };
        let max_number = max_number_opt.unwrap();
        let (min_number, mut has_dirty) = {
            let dirty_number = self
                .select_checked_number(&lock_hash.as_bytes())?
                .unwrap_or(0);
            let checked_number = self
                .select_checked_number(&lock_hash.as_bytes())?
                .unwrap_or(0);
            if dirty_number > checked_number {
                (dirty_number, true)
            } else {
                (checked_number, false)
            }
        };
        for num in min_number..max_number {
            let block = self.select_block_by_number(num)?.unwrap();
            for tx in block.transactions.into_iter() {
                for (idx, cell) in tx.inner.outputs.into_iter().enumerate() {
                    if &cell.lock == lock_script {
                        if let Some(confirm) = self.select_cell_status(&tx.hash, idx as u32)? {
                            if confirm {
                                self.update_cell_status(&tx.hash, idx as u32, false)?;
                                let outpoint = types::CellOutPoint {
                                    tx_hash: tx.hash,
                                    index: types::Unsigned(idx as u64),
                                };
                                return Ok(Some((outpoint, cell)));
                            }
                        } else {
                            has_dirty = true;
                        }
                    }
                }
            }
            if has_dirty {
                self.insert_dirty_number(&lock_hash.as_bytes(), num)?;
            } else {
                self.insert_checked_number(&lock_hash.as_bytes(), num)?;
            }
        }
        Ok(None)
    }
}
