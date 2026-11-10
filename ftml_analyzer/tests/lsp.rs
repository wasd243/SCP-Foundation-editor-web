use ftml_analyzer::debug_parse;
use std::fs;

const FTML: &str = "tests/ftmls/";
const OUTPUT_PATH: &str = "tests/abs_tokens/";

#[cfg(feature = "bless")]
#[test]
fn bless_highlight_abs_token() {
    for entry in fs::read_dir(FTML).unwrap() {
        let path = entry.unwrap().path();

        if !path.is_file() {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap();

        let stem = path.file_stem().unwrap().to_string_lossy();

        let abs_token = debug_parse(&content).unwrap();

        let path = format!("{OUTPUT_PATH}{stem}.json");

        fs::write(path, abs_token).unwrap();
    }
}

#[test]
fn check_highlight_abs_token() {
    for entry in fs::read_dir(FTML).unwrap() {
        let path = entry.unwrap().path();

        if !path.is_file() {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap();

        let stem = path.file_stem().unwrap().to_string_lossy();

        let abs_token = debug_parse(&content).unwrap();

        let path = format!("{OUTPUT_PATH}{stem}.json");

        let expect = fs::read_to_string(path).unwrap();

        assert_eq!(expect, abs_token);
    }
}
