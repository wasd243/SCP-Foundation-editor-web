use ftml_analyzer::debug_parse;
use std::{
    fs,
    path::{Path, PathBuf},
};

const FTML: &str = "tests/ftmls";
const OUTPUT_PATH: &str = "tests/abs_tokens";

fn collect_ftml(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();

        if path.is_dir() {
            collect_ftml(&path, out);
        } else if path.extension().is_some_and(|e| e == "ftml") {
            out.push(path);
        }
    }
}

/// `tests/ftmls/edge_cases/1.ftml` -> `tests/abs_tokens/edge_cases/1.json`
fn output_path_of(ftml_path: &Path) -> PathBuf {
    let relative = ftml_path.strip_prefix(FTML).unwrap();
    Path::new(OUTPUT_PATH).join(relative).with_extension("json")
}

fn all_ftml_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_ftml(Path::new(FTML), &mut files);
    files.sort();
    files
}

#[cfg(feature = "bless")]
#[test]
fn bless_highlight_abs_token() {
    for path in all_ftml_files() {
        let content = fs::read_to_string(&path).unwrap();
        let abs_token = debug_parse(&content).unwrap();

        fs::write(output_path_of(&path), abs_token).unwrap();
    }
}

#[test]
fn check_highlight_abs_token() {
    for path in all_ftml_files() {
        let content = fs::read_to_string(&path).unwrap();
        let abs_token = debug_parse(&content).unwrap();

        let expect_path = output_path_of(&path);
        let expect = fs::read_to_string(&expect_path)
            .unwrap_or_else(|e| panic!("missing expected file {}: {e}", expect_path.display()));

        assert_eq!(expect, abs_token, "mismatch for {}", path.display());
    }
}
