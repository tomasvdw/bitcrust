extern crate store;

extern crate serde_json;

mod util;


#[test]
fn test_get() {


    let db = &mut store::init("tst-import").unwrap();

    let hdr = store::header_get(db,
        &util::hash_from_hex("000000000000034a7dedef4a161fa058a2d67a173a90155f3a2fe6fc132e0ebf"))
        .unwrap().unwrap();

    let tx = store::transaction_get(db,
                                &util::hash_from_hex("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"))
        .unwrap().unwrap();

    println!("{}", serde_json::to_string_pretty(&hdr.header).unwrap());
    println!("{}", serde_json::to_string_pretty(&tx.as_tx().unwrap()).unwrap());
}

#[test]
fn test_get_all_headers() {
    // just browse through all imported headers;
    let db = &mut store::init("tst-import").unwrap();

    let mut hash = store::header_get_best(db).unwrap();

    println!("{:?}", hash);
    loop {
        let hdr = store::header_get(db, &hash).unwrap().unwrap();

        if hdr.height == 0 {
            break;
        }

        hash = hdr.header.prev_hash;

    }
}

#[test]
fn test_locator() {
    // just browse through all imported headers;
    let db = &mut store::init("tst-import").unwrap();

    let mut hash = store::header_get_best(db).unwrap();

    println!("{:?}", hash);
    loop {
        let hdr = store::header_get(db, &hash).unwrap().unwrap();

        if hdr.height == 0 {
            break;
        }

        hash = hdr.header.prev_hash;

    }
}






fn main() {
    println!("Hallo");
}
