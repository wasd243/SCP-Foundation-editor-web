use ftml_handler::ftml_ast::{ParseToken, ParseTokenTree};
use std::fs;
use std::path::PathBuf;

const BLESS: bool = cfg!(feature = "bless");
const FTML: &str = "tests/ftmls/";

#[test]
fn bless_token() {
    if !BLESS {
        return;
    }

    let path = PathBuf::from(FTML);
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let content = fs::read_to_string(&path).unwrap();

            let mut token = ParseToken::from(content.as_str());
            token.clean();
            token.mark_blocks();
            let token = serde_json::to_string_pretty(&token).unwrap();

            let stem = path.file_stem().unwrap().to_string_lossy();

            let path = format!("tests/tokens/{stem}.json");
            fs::write(path, token).unwrap()
        }
    }
}

#[test]
fn bless_token_trees() {
    if !BLESS {
        return;
    }

    let path = PathBuf::from(FTML);
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let content = fs::read_to_string(&path).unwrap();

            let mut token = ParseToken::from(content.as_str());
            token.clean();
            token.mark_blocks();

            let token = ParseTokenTree::from_parse_tokens(token);

            let token = serde_json::to_string_pretty(&token).unwrap();

            let stem = path.file_stem().unwrap().to_string_lossy();

            let path = format!("tests/token_trees/{stem}.json");
            fs::write(path, token).unwrap()
        }
    }
}

#[test]
fn check_token() {
    if BLESS {
        return;
    }

    let path = PathBuf::from(FTML);
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let content = fs::read_to_string(&path).unwrap();

            let mut token = ParseToken::from(content.as_str());
            token.clean();
            token.mark_blocks();
            let token = serde_json::to_string_pretty(&token).unwrap();

            let stem = path.file_stem().unwrap().to_string_lossy();

            let path = format!("tests/tokens/{stem}.json");

            let expect = fs::read_to_string(path).unwrap();

            assert_eq!(expect, token);
        }
    }
}

#[test]
fn check_token_trees() {
    if BLESS {
        return;
    }

    let path = PathBuf::from(FTML);
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let content = fs::read_to_string(&path).unwrap();

            let mut token = ParseToken::from(content.as_str());
            token.clean();
            token.mark_blocks();

            let token = ParseTokenTree::from_parse_tokens(token);

            let token = serde_json::to_string_pretty(&token).unwrap();

            let stem = path.file_stem().unwrap().to_string_lossy();

            let path = format!("tests/token_trees/{stem}.json");

            let expect = fs::read_to_string(path).unwrap();

            assert_eq!(expect, token);
        }
    }
}
