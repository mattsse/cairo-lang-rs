fn main() {
    // lalrpop::process_root().unwrap();
    let dir = format!("{}/src", std::env::var("CARGO_MANIFEST_DIR").unwrap());
    lalrpop::Configuration::new().process_dir(dir).unwrap();
}
