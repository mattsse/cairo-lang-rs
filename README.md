cairo-lang-rs
=========================

[<img alt="github" src="https://img.shields.io/badge/github-mattsse/cairo-lang-rs-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/mattsse/cairo-lang-rs)
[<img alt="crates.io" src="https://img.shields.io/crates/v/cairo-lang-rs.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/cairo-lang-rs)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-cairo-lang-rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/cairo-lang-rs)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/mattsse/cairo-lang-rs/CI/main?style=for-the-badge" height="20">](https://github.com/mattsse/cairo-lang-rs/actions?query=branch%3Amain)

Rust support for the [Cairo](https://www.cairo-lang.org/) programming language.

WIP...

# Usage

Parse the content of a cairo file

```rust
use cairo_lang::CairoFile;

let instructions = CairoFile::read("file.cairo").unwrap();
```

Licensed under either of these:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)
* All *.cairo files in [./common/](./common) are taken from [starkware-libs/cairo-lang](https://github.com/starkware-libs/cairo-lang) and are thus licensed under the [Cairo Toolchain License](https://raw.githubusercontent.com/starkware-libs/cairo-lang/master/LICENSE.txt)

