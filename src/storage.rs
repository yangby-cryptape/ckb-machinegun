// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{convert::TryInto, path::Path, sync::Arc};

use parking_lot::Mutex;
use rusqlite::{Connection, Result, NO_PARAMS};

use ckb_jsonrpc_interfaces::{core, types, H256};

pub(crate) struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    pub(crate) fn open<P>(file: P) -> Result<Arc<Self>>
    where
        P: AsRef<Path>,
    {
        let conn = Connection::open(file)?;
        conn.set_prepared_statement_cache_capacity(0);
        Ok(Arc::new(Self {
            conn: Mutex::new(conn),
        }))
    }

    fn execute<T, F>(&self, func: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let wait_time = ::std::time::Duration::from_secs(60 * 60 * 24);
        let conn = self.conn.try_lock_for(wait_time).unwrap();
        func(&conn)
    }

    pub(crate) fn init(&self) -> Result<()> {
        self.execute(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS
                    wallet_status
                (
                    key         CHAR(16)    PRIMARY KEY     NOT NULL,
                    number      INTEGER                     NOT NULL
                );



                CREATE TABLE IF NOT EXISTS
                    chain_status
                (
                    number      INTEGER     PRIMARY KEY     NOT NULL,
                    timestamp   INTEGER                     NOT NULL
                );
                CREATE TABLE IF NOT EXISTS
                    chain_transactions
                (
                    hash        CHAR(64)    PRIMARY KEY     NOT NULL,
                    number      INTEGER                     NOT NULL
                );
                CREATE INDEX IF NOT EXISTS
                    number_on_chain_transactions
                ON
                    chain_transactions (number);
                CREATE TABLE IF NOT EXISTS
                    chain_cells
                (
                    tx_hash     CHAR(64)                    NOT NULL,
                    index_      INTEGER                     NOT NULL,
                    capacity    INTEGER                     NOT NULL,
                    PRIMARY KEY (tx_hash, index_)
                );



                CREATE TABLE IF NOT EXISTS
                    stolen_transactions
                (
                    hash        CHAR(64)    PRIMARY KEY     NOT NULL,
                    number      INTEGER                     NOT NULL
                );
                CREATE INDEX IF NOT EXISTS
                    number_on_stolen_transactions
                ON
                    stolen_transactions (number);
                CREATE TABLE IF NOT EXISTS
                    transactions
                (
                    tx_hash     CHAR(64)                    NOT NULL,
                    index_      INTEGER                     NOT NULL,
                    hash        CHAR(64)                    NOT NULL,
                    number      INTEGER                     NOT NULL,
                    PRIMARY KEY (tx_hash, index_)
                );
                CREATE INDEX IF NOT EXISTS
                    hash_on_transactions
                ON
                    transactions (hash);
                CREATE INDEX IF NOT EXISTS
                    number_on_transactions
                ON
                    transactions (number);
            "#,
            )
        })
    }

    fn update_wallet_status(&self, key: &str, number: core::BlockNumber) -> Result<()> {
        assert!(key.len() < 16);
        let stmt = r#"INSERT OR REPLACE INTO wallet_status (key, number) VALUES (:key, :number);"#;
        self.execute(|conn| {
            conn.prepare(stmt)?
                .execute_named(&[(":key", &key), (":number", &(number as i64))])
                .map(|_| ())
        })
    }

    fn select_wallet_status(&self, key: &str) -> Result<core::BlockNumber> {
        let stmt = "SELECT ifnull(max(number), 0) from wallet_status where key = :key;";
        self.execute(|conn| {
            conn.query_row::<i64, _, _>(stmt, &[key], |r| r.get(0))
                .map(|x| x as core::BlockNumber)
        })
    }

    pub(crate) fn update_tip_status(&self, number: core::BlockNumber) -> Result<()> {
        self.update_wallet_status("tip", number)
    }

    pub(crate) fn select_tip_status(&self) -> Result<core::BlockNumber> {
        self.select_wallet_status("tip")
    }

    pub(crate) fn update_turn_status(&self, number: core::BlockNumber) -> Result<()> {
        self.update_wallet_status("turn", number)
    }

    pub(crate) fn select_turn_status(&self) -> Result<core::BlockNumber> {
        self.select_wallet_status("turn")
    }

    pub(crate) fn update_stolen_status(&self, number: core::BlockNumber) -> Result<()> {
        self.update_wallet_status("stolen", number)
    }

    pub(crate) fn select_stolen_status(&self) -> Result<core::BlockNumber> {
        self.select_wallet_status("stolen")
    }

    pub(crate) fn update_chain_status(
        &self,
        number: core::BlockNumber,
        timestamp: u64,
    ) -> Result<()> {
        let stmt = r#"
            INSERT OR REPLACE INTO
                chain_status (number, timestamp)
            VALUES (:number, :timestamp);"#;
        self.execute(|conn| {
            conn.prepare(stmt)?
                .execute_named(&[
                    (":number", &(number as i64)),
                    (":timestamp", &(timestamp as i64)),
                ])
                .map(|_| ())
        })
    }

    pub(crate) fn select_chain_number(&self) -> Result<Option<core::BlockNumber>> {
        let stmt = "SELECT max(number) from chain_status";
        self.execute(|conn| {
            conn.query_row::<Option<i64>, _, _>(stmt, NO_PARAMS, |r| r.get(0))
                .map(|x| x.map(|y| y as core::BlockNumber))
        })
    }

    pub(crate) fn save_chain_transactions(
        &self,
        number: core::BlockNumber,
        txs: &[H256],
    ) -> Result<()> {
        let stmt = r#"
            INSERT OR IGNORE INTO
                chain_transactions (hash, number)
            VALUES (:hash, :number);"#;
        self.execute(move |conn| {
            let mut stmt = conn.prepare(stmt)?;
            for h in txs.iter() {
                let hash = format!("{:x}", h);
                stmt.execute_named(&[(":hash", &hash), (":number", &(number as i64))])
                    .unwrap();
            }
            Ok(())
        })
    }

    pub(crate) fn save_chain_cells(&self, cells: &[(H256, u32, u64)]) -> Result<()> {
        let stmt = r#"
            INSERT OR IGNORE INTO
                chain_cells
            (
                tx_hash, index_, capacity
            ) VALUES (
                :tx_hash, :index_, :capacity
            );"#;
        self.execute(|conn| {
            let mut stmt = conn.prepare(stmt)?;
            for cell in cells.iter() {
                let tx_hash = format!("{:x}", cell.0);
                stmt.execute_named(&[
                    (":tx_hash", &tx_hash),
                    (":index_", &cell.1),
                    (":capacity", &(cell.2 as i64)),
                ])
                .unwrap();
            }
            Ok(())
        })
    }

    pub(crate) fn save_stolen_transaction(
        &self,
        number: core::BlockNumber,
        tx_hash: H256,
    ) -> Result<()> {
        let stmt = r#"
            INSERT OR IGNORE INTO
                stolen_transactions (hash, number)
            VALUES (:hash, :number);"#;
        let hash = format!("{:x}", tx_hash);
        self.execute(move |conn| {
            conn.prepare(stmt)?
                .execute_named(&[(":hash", &hash), (":number", &(number as i64))])
                .map(|_| ())
        })
    }

    pub(crate) fn save_transaction(&self, tx: (H256, u32, H256)) -> Result<()> {
        let stmt = r#"
            INSERT OR IGNORE INTO
                transactions
            (
                tx_hash, index_, hash, number
            )
            SELECT :tx_hash, :index_, :hash, number
              FROM wallet_status
             WHERE key = "tip"
             LIMIT 1
            ;"#;
        let tx_hash = format!("{:x}", tx.0);
        let index = tx.1;
        let hash = format!("{:x}", tx.2);
        self.execute(move |conn| {
            conn.prepare(stmt)?
                .execute_named(&[
                    (":tx_hash", &tx_hash),
                    (":index_", &index),
                    (":hash", &hash),
                ])
                .map(|_| ())
        })
    }

    pub(crate) fn fetch_unspent_cells(
        &self,
        limit: u32,
    ) -> Result<Vec<(core::transaction::CellInput, u64)>> {
        let stmt = r#"
            SELECT cc.tx_hash, cc.index_, cc.capacity
              FROM stolen_transactions st
         LEFT JOIN chain_cells cc
                ON st.hash = cc.tx_hash
             WHERE cc.capacity is not null
               AND NOT EXISTS (
                SELECT 1
                  FROM transactions t
                 WHERE cc.tx_hash = t.tx_hash
                   AND cc.index_ = t.index_)
             LIMIT :limit;"#;
        self.execute(|conn| {
            let mut stmt = conn.prepare(stmt)?;
            let mut rows = stmt.query_named(&[(":limit", &limit)])?;
            let mut inputs = Vec::new();
            while let Some(row) = rows.next()? {
                let h: String = row.get(0).unwrap();
                let tx_hash = H256::from_hex_str(&h[..]).unwrap();
                let index = row.get(1).unwrap();
                let capacity: i64 = row.get(2).unwrap();
                let out_point = types::OutPoint { tx_hash, index };
                let input = core::transaction::CellInput {
                    previous_output: out_point.try_into().unwrap(),
                    since: 0,
                    args: vec![],
                };
                inputs.push((input, capacity as u64));
            }
            Ok(inputs)
        })
    }

    pub(crate) fn count_unspent_cells(&self) -> Result<u64> {
        let stmt = r#"
            SELECT count(1)
              FROM stolen_transactions st
         LEFT JOIN chain_cells cc
                ON st.hash = cc.tx_hash
             WHERE cc.capacity is not null
               AND NOT EXISTS (
                SELECT 1
                  FROM transactions t
                 WHERE cc.tx_hash = t.tx_hash
                   AND cc.index_ = t.index_);"#;
        self.execute(|conn| {
            conn.query_row::<i64, _, _>(stmt, NO_PARAMS, |r| r.get(0))
                .map(|x| x as u64)
        })
    }

    pub(crate) fn count_transactions(&self) -> Result<(u64, u64, u64)> {
        let stmt = r#"
            SELECT count(1), count(t.hash), count(ct.number)
              FROM transactions t
         LEFT JOIN chain_transactions ct
                ON t.hash = ct.hash;"#;
        self.execute(|conn| {
            conn.query_row::<(i64, i64, i64), _, _>(stmt, NO_PARAMS, |r| {
                r.get(0)
                    .and_then(|x| r.get(1).and_then(|y| r.get(2).map(|z| (x, y, z))))
            })
            .map(|(x, y, z)| (x as u64, y as u64, z as u64))
        })
    }

    pub(crate) fn do_statistics(
        &self,
        number: core::BlockNumber,
    ) -> Result<(u64, u64, u64, u64, u64, u64)> {
        let stmt = r#"
                SELECT count(1),
                       ifnull(min(cs1.timestamp), 0),
                       ifnull(max(cs2.timestamp), 0),
                       ifnull(min(cs2.timestamp - cs1.timestamp), 0),
                       ifnull(max(cs2.timestamp - cs1.timestamp), 0),
                       ifnull(sum(cs2.timestamp - cs1.timestamp), 0)
                  FROM transactions t
             LEFT JOIN chain_transactions ct
                    ON t.hash = ct.hash
             LEFT JOIN chain_status cs1
                    ON t.number = cs1.number
             LEFT JOIN chain_status cs2
                    ON ct.number = cs2.number
                 WHERE 1 = 1
                   AND t.hash is not null       -- tx sent
                   AND ct.hash is not null      -- tx committed
                   AND t.number >= :number
                ;"#;
        self.execute(|conn| {
            conn.query_row::<(i64, i64, i64, i64, i64, i64), _, _>(stmt, &[&(number as i64)], |r| {
                let cnt = r.get(0)?;
                let start = r.get(1)?;
                let end = r.get(2)?;
                let min = r.get(3)?;
                let max = r.get(4)?;
                let sum = r.get(5)?;
                Ok((cnt, start, end, min, max, sum))
            })
            .map(|(a, b, c, d, e, f)| (a as u64, b as u64, c as u64, d as u64, e as u64, f as u64))
        })
    }
}
