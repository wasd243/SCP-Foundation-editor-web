//! This file contains some manual implemented tests, so it's not placed in `tests` crate.

macro_rules! cursor {
    ($from:ident, $output:ident) => {
        let tokens = ::ftml_analyzer::ftml_ast::ParseToken::from($from);
        let mut $output = ::ftml_analyzer::Cursor::new(&tokens.tokens).unwrap();
    };
}

/// Assert one value per token, advancing after each assertion.
macro_rules! cursor_next {
     ($cursor:ident, $method:ident, $($expected:expr),+ $(,)?) => {
         $(
            assert_eq!($cursor.$method(), $expected, "token: {:?}", $cursor.token.ty);
            $cursor.next();
        )+
    };
}

const FTML: &str = "[[footnote]]something[[/footnote]]";
const FTML2: &str = r#"[[collapsible show="" hide=""]]"#;
const FTML3: &str = r#"[[div class="cls"]]"#;

#[test]
fn cursor_works() {
    cursor!(FTML, cursor);
    // [[  footnote  ]]  something
    cursor_next!(cursor, is_in_block, true, true, true, false);
}

#[test]
fn value_correct() {
    cursor!(FTML2, cursor);
    // fill in one bool per token, then run once and fix per the failure message
    cursor_next!(
        cursor, is_value, false, false, false, false, false, false, false, false, false
    );
}

#[test]
fn string_correct() {
    cursor!(FTML3, cursor);
    cursor_next!(
        cursor, is_string, false, false, false, false, false, true, true, true, false
    );
}

#[test]
fn dump_string() {
    cursor!(FTML3, cursor);
    loop {
        dbg!(cursor.token.ty, cursor.is_string(), cursor.is_value());
        if cursor.next().is_none() {
            break;
        }
    }
}
