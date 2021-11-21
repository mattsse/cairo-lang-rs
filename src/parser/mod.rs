pub mod ast;
pub mod error;
pub mod lexer;

pub use grammar::*;

mod grammar {
    #![allow(dead_code)]
    #![allow(clippy::all)]
    lalrpop_util::lalrpop_mod!(pub cairo_grammar, "/parser/cairo_grammar.rs");
}

#[cfg(test)]
mod tests {

    use crate::{parser::ast::*, parser::lexer::*};
    use std::path::Path;

    fn tokenize(s: &str) -> Vec<Result<(usize, CairoToken, usize), CairoLexerError>> {
        CairoLexer::new(s).collect()
    }

    macro_rules! _parse {
        ($input:expr, $parser:ident) => {
            cairo_grammar::$parser::new().parse($input, CairoLexer::new($input))
        };
    }

    macro_rules! _assert_parse {
        ($input:expr, $parser:ident) => {
            assert!(parse!($input, $parser).is_ok());
        };
        ($input:expr, [$parser1:ident, $parser2:ident]) => {
            assert!(parse!($input, $parser1).is_ok());
            assert!(parse!($input, $parser2).is_ok());
        };
    }

    macro_rules! _parse_unwrap {
        ($input:expr, $parser:ident) => {
            cairo_grammar::$parser::new()
                .parse($input, CairoLexer::new($input))
                .unwrap();
        };
        ($input:expr, [$parser1:ident, $parser2:ident]) => {
            cairo_grammar::$parser1::new()
                .parse($input, CairoLexer::new($input))
                .unwrap();
            cairo_grammar::$parser2::new()
                .parse($input, CairoLexer::new($input))
                .unwrap();
        };
    }

    fn parse_files_in_folder(dir: impl AsRef<Path>) {
        for file in
            std::fs::read_dir(Path::new(&env!("CARGO_MANIFEST_DIR")).join(dir.as_ref())).unwrap()
        {
            let file = file.unwrap();
            let file_name = format!("{}", file.path().file_name().unwrap().to_string_lossy());
            let content = std::fs::read_to_string(file.path()).unwrap();
            assert!(
                tokenize(&content).into_iter().all(|x| x.is_ok()),
                "{}",
                file_name
            );
            CairoFile::parse(&content).expect(&file_name);
        }
    }

    // /// Ensure we can parse all common cairo files from cairo-lang
    // #[test]
    // fn parse_common_cairo_files() {
    //     parse_files_in_folder("common")
    // }
    //
    // #[test]
    // fn parse_testdata_cairo_files() {
    //     parse_files_in_folder("test-data/cairo-files")
    // }

    #[test]
    fn was() {
        assert!(true);
    }
}
