fn main() {
    lalrpop::Configuration::new()
        .emit_report(true)
        .generate_in_source_tree()
        .process_dir("src/lalrpop/grammar").unwrap()
}
