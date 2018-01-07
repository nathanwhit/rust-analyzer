extern crate file;
#[macro_use(assert_diff)]
extern crate difference;
extern crate libsyntax2;

use std::path::{PathBuf, Path};
use std::fs::read_dir;
use std::fmt::Write;

use libsyntax2::{tokenize, parse, Node, File};

#[test]
fn parser_tests() {
    for test_case in parser_test_cases() {
        parser_test_case(&test_case);
    }
}

fn parser_test_dir() -> PathBuf {
    let dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(dir).join("tests/data/parser")
}

fn test_from_dir(dir: &Path) -> Vec<PathBuf> {
    let mut acc = Vec::new();
    for file in read_dir(&dir).unwrap() {
        let file = file.unwrap();
        let path = file.path();
        if path.extension().unwrap_or_default() == "rs" {
            acc.push(path);
        }
    }
    acc.sort();
    acc
}

fn parser_test_cases() -> Vec<PathBuf> {
    let mut acc = Vec::new();
    acc.extend(test_from_dir(&parser_test_dir().join("ok")));
    acc.extend(test_from_dir(&parser_test_dir().join("err")));
    acc
}

fn parser_test_case(path: &Path) {
    let actual = {
        let text = file::get_text(path).unwrap();
        let tokens = tokenize(&text);
        let file = parse(text, &tokens);
        dump_tree(&file)
    };
    let expected = path.with_extension("txt");
    let expected = file::get_text(&expected).expect(
        &format!("Can't read {}", expected.display())
    );
    let expected = expected.as_str();
    let actual = actual.as_str();
    if expected == actual {
        return
    }
    if expected.trim() == actual.trim() {
        panic!("Whitespace difference! {}", path.display())
    }
    assert_diff!(expected, actual, "\n", 0)
}

fn dump_tree(file: &File) -> String {
    let mut result = String::new();
    go(file.root(), &mut result, 0);
    return result;

    fn go(node: Node, buff: &mut String, level: usize) {
        buff.push_str(&String::from("  ").repeat(level));
        write!(buff, "{:?}\n", node).unwrap();
        let my_errors = node.errors().filter(|e| e.after_child().is_none());
        let parent_errors = node.parent().into_iter()
            .flat_map(|n| n.errors())
            .filter(|e| e.after_child() == Some(node));

        for err in my_errors {
            buff.push_str(&String::from("  ").repeat(level));
            write!(buff, "err: `{}`\n", err.message()).unwrap();
        }

        for child in node.children() {
            go(child, buff, level + 1)
        }

        for err in parent_errors {
            buff.push_str(&String::from("  ").repeat(level));
            write!(buff, "err: `{}`\n", err.message()).unwrap();
        }
    }
}
