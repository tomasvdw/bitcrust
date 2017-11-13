extern crate store;
mod util;

fn header_to_json(hdr: &store::Header) -> String {
    format!(
"{{\
  version: {}\
}}", hdr.version)

}


#[test]
fn test_get() {


    let mut db = store::init("tst-import").unwrap();

    let hdr = store::header_get(&mut db,
        &util::hash_from_hex("000000000000034a7dedef4a161fa058a2d67a173a90155f3a2fe6fc132e0ebf"))
        .unwrap().unwrap();

    println!("{}", header_to_json(&hdr));
}









fn main() {
    println!("Hallo");
}
