

extern crate store;
extern crate serde_json;
mod util;

fn hash_from_slice(slice: &[u8]) -> [u8;32] {
    let mut result = [0;32];
    result.copy_from_slice(&slice[0..32]);
    result
}


#[test]
fn test_empty() {
    let mut db = store::init_empty("tst-empty").unwrap();

    // get genesis coinbase tx
    let dbtx = store::transaction_get(&mut db, &hash_from_slice(&util::from_hex_rev(
        "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b")
    )).unwrap().unwrap();

    let tx = dbtx.as_tx().unwrap();
    println!("{}", serde_json::to_string_pretty(&tx).unwrap());
    assert_eq!(tx.version, 1);
    assert_eq!(tx.txs_out[0].value, 50 * 100_000_000);
}

//10mb/sec => 13000