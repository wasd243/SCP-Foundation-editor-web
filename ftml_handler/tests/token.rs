use ftml_analyzer::ftml_ast::ParseToken;
use ftml_analyzer::lsp_builtin::LspBuiltIn;
use serde_json::to_string_pretty;
use std::fs;
use std::path::PathBuf;

const BLESS: bool = cfg!(feature = "bless");
const FTML: &str = "tests/ftmls/";

fn test(input_path: &str, output_path: &str, need_visitor: bool, is_bless: bool) {
    // bless tests only run with `--features bless`, check tests only run without it.
    if is_bless != BLESS {
        return;
    }

    for entry in fs::read_dir(input_path).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap();

        let mut token = ParseToken::from(content.as_str());
        token.clean();
        token.mute_ftml_fps();

        let (out_path, json) = get_path_and_token(output_path, &path, &token, need_visitor);

        if is_bless {
            fs::write(out_path, json).unwrap();
        } else {
            let expect = fs::read_to_string(out_path).unwrap();
            assert_eq!(expect, json);
        }
    }
}

fn get_path_and_token(
    output_path: &str,
    file_path: &PathBuf,
    parse_token: &ParseToken,
    need_visitor: bool,
) -> (String, String) {
    if need_visitor {
        let mut token = LspBuiltIn::visitor(parse_token);
        token.sanitize_whitespace();

        let token = to_string_pretty(&token).unwrap();

        let stem = file_path.file_stem().unwrap().to_string_lossy();

        let path = format!("tests/{output_path}/{stem}.json");

        (path, token)
    } else {
        let token = to_string_pretty(parse_token).unwrap();
        let stem = file_path.file_stem().unwrap().to_string_lossy();
        let path = format!("tests/{output_path}/{stem}.json");
        (path, token)
    }
}

#[test]
fn bless_builtin() {
    test(FTML, "builtin", true, true);
}

#[test]
fn bless_token() {
    test(FTML, "tokens", false, true);
}

#[test]
fn check_builtin() {
    test(FTML, "builtin", true, false);
}

#[test]
fn check_token() {
    test(FTML, "tokens", false, false);
}
