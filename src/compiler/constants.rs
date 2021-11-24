use ethers::types::U256;

pub const LIBS_DIR_ENVVAR: &str = "CAIRO_PATH";

pub const START_FILE_NAME: &str = "<start>";

pub const CAIRO_FILE_EXTENSION: &str = ".cairo";

pub const N_LOCALS_CONSTANT: &str = "SIZEOF_LOCALS";

pub const ARG_SCOPE: &str = "Args";
pub const IMPLICIT_ARG_SCOPE: &str = "ImplicitArgs";

pub const RETURN_SCOPE: &str = "Return";

// 2 ** 251 + 17 * 2 ** 192 + 1
pub const DEFAULT_PRIME: U256 = U256([1, 0, 0, 576460752303423505]);

pub const START_CODE: &str = r#"
__start__:
ap += main.Args.SIZE + main.ImplicitArgs.SIZE
call main

__end__:
jmp rel 0
"#;

#[test]
fn prime() {
    let prime: U256 = U256::from_dec_str(
        "3618502788666131213697322783095070105623107215331596699973092056135872020481",
    )
    .unwrap();
    assert_eq!(prime, DEFAULT_PRIME);
}
