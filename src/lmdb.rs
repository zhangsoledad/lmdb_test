use libmdb;
use bincode::{deserialize, serialize, Infinite};
use libmdb::{Cursor, Database, DatabaseFlags, Environment, Transaction, WriteFlags};
use std::path::Path;

const height_key: [u8; 1] = [104];
const head_key: [u8; 1] = [105];
const tail_key: [u8; 1] = [116];

#[derive(Debug)]
pub struct LMDB {
    pub env: Environment,
    pub main: Database,
    pub index: Database,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, Clone)]
pub struct Num(pub u64);

impl Default for Num {
    fn default() -> Num {
        Num(0)
    }
}

impl<'a> From<&'a [u8]> for Num {
    fn from(bytes: &[u8]) -> Num {
        deserialize(bytes).unwrap()
    }
}

impl LMDB {
    // add code here
    pub fn new(path: &Path, size: usize) -> Self {
        let mut builder = libmdb::Environment::new();
        builder.set_max_dbs(2);
        builder.set_map_size(size);
        let env = builder.open(path).expect("open lmdb env");
        let index = env.create_db(Some("index"), libmdb::DUP_SORT)
            .expect("open index db");
        let main = env.create_db(Some("main"), DatabaseFlags::empty())
            .expect("open main db");

        LMDB {
            env: env,
            main: main,
            index: index,
        }
    }


    pub fn clear(&self) -> Result<(), libmdb::Error> {
        let mut txn = self.env.begin_rw_txn()?;
        txn.clear_db(self.main)?;
        txn.clear_db(self.index)?;
        txn.commit()
    }


    pub fn save_height(&self, height: u64) -> Result<(), libmdb::Error> {
        let mut tx = self.env.begin_rw_txn()?;
        let height = Num(height);
        let encoded_height = serialize(&height, Infinite).unwrap();
        let new_height = tx.put(
            self.index,
            &height_key,
            &encoded_height.as_slice(),
            libmdb::NO_DUP_DATA,
        );
        if new_height.is_ok() {
            let head = tx.get(self.main, &head_key)
                .map(From::from)
                .unwrap_or_else(|_| Num::default());
            if height > head {
                tx.put(
                    self.main,
                    &head_key,
                    &encoded_height.clone().as_slice(),
                    WriteFlags::empty(),
                )?;
            }
            if let Err(libmdb::Error::NotFound) = tx.get(self.main, &tail_key) {
                tx.put(
                    self.main,
                    &tail_key,
                    &encoded_height.clone().as_slice(),
                    WriteFlags::empty(),
                )?;
            }
        }
        tx.commit()
    }
}
