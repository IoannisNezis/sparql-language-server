use std::{
    fs::{self, File},
    io::Read,
    process::Command,
};

use tree_sitter::Parser;

#[test]
fn compare_parse_trees() {
    let mut unchanged = 0;
    let mut changed = 0;
    for entry in fs::read_dir("tests/example-input/").unwrap() {
        let path = entry.unwrap().path();
        let mut file = File::open(path.as_path()).expect("File should be readable");
        let mut content_before = String::new();
        file.read_to_string(&mut content_before).unwrap();
        let output = qlue_ls::format_raw(content_before.clone()).expect("Formatting should work");
        let content_after = String::from_utf8_lossy(&output.as_bytes()).to_string();

        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_sparql::LANGUAGE.into())
            .unwrap();
        let tree_before = parser.parse(content_before, None).unwrap();
        let tree_after = parser.parse(content_after, None).unwrap();
        if tree_before.root_node().to_string() != tree_after.root_node().to_string() {
            println!("Query changed by format: {}", path.to_str().unwrap());
            changed += 1;
        } else {
            unchanged += 1;
        }
    }
    println!(
        "----------RESULT-----------\n{}/{} failed\n-----------------------------",
        changed,
        changed + unchanged
    );
    assert!(changed == 0);
}
