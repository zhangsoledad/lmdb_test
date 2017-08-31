extern crate bincode;
extern crate lmdb as libmdb;
extern crate rayon;
#[macro_use]
extern crate serde_derive;
extern crate tempdir;

mod lmdb;

use tempdir::TempDir;
use rayon::prelude::*;
use lmdb::LMDB;

fn main() {
    let dir = TempDir::new("test").unwrap();
    let db = LMDB::new(dir.path(), 1_000_000_000);

    db.clear();

    let height: Vec<_> = (1..100000).collect();

    height
        .iter()
        .map(|height| { db.save_height(*height); })
        .collect::<Vec<()>>();
}
