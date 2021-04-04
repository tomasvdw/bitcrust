fn main() {
    println!("cargo:rustc-link-search=/usr/local/Cellar/bitcoin/0.21.0/lib\n\
    cargo:rustc-link-lib=dylib=bitcoinconsensus");
}