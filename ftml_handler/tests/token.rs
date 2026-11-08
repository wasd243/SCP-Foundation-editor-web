use ftml_analyzer::ftml_ast::ParseToken;
use ftml_analyzer::lsp_builtin::LspBuiltIn;
use serde_json::to_string_pretty;
use std::fs;
use std::path::PathBuf;

const BLESS: bool = cfg!(feature = "bless");
const FTML: &str = "tests/ftmls/";

macro_rules! bless {
    ($input_path:expr, $output_path:expr, $need_visitor:expr) => {
        if !BLESS {
            return;
        }

        let output_dir = $output_path;

        for entry in fs::read_dir(PathBuf::from($input_path)).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let content = fs::read_to_string(&path).unwrap();

                let mut token = ParseToken::from(content.as_str());
                token.clean();
                token.mute_ftml_fps();

                if $need_visitor {
                    let token = LspBuiltIn::visitor(&token);
                    let token = to_string_pretty(&token).unwrap();

                    let stem = path.file_stem().unwrap().to_string_lossy();

                    let path = format!("tests/{output_dir}/{stem}.json");
                    fs::write(path, token).unwrap()
                } else {
                    let token = to_string_pretty(&token).unwrap();

                    let stem = path.file_stem().unwrap().to_string_lossy();

                    let path = format!("tests/{output_dir}/{stem}.json");
                    fs::write(path, token).unwrap()
                }
            }
        }
    };
}

#[test]
fn bless_builtin() {
    bless!(FTML, "builtin", true);
}

#[test]
fn bless_token() {
    bless!(FTML, "tokens", false);
}

#[test]
fn check_builtin() {
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
            token.mute_ftml_fps();

            let token = LspBuiltIn::visitor(&token);

            let token = to_string_pretty(&token).unwrap();

            let stem = path.file_stem().unwrap().to_string_lossy();

            let path = format!("tests/builtin/{stem}.json");

            let expect = fs::read_to_string(path).unwrap();

            assert_eq!(expect, token);
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
            token.mute_ftml_fps();

            let token = to_string_pretty(&token).unwrap();

            let stem = path.file_stem().unwrap().to_string_lossy();

            let path = format!("tests/tokens/{stem}.json");

            let expect = fs::read_to_string(path).unwrap();

            assert_eq!(expect, token);
        }
    }
}
