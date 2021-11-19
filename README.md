cairo-lang-rs
=========================

[<img alt="github" src="https://img.shields.io/badge/github-mattsse/cairo-lang-rs-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/mattsse/cairo-lang-rs)
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
* All *.cairo files in [./common/](./common) and [./test-data/](./test-data) are taken from [starkware-libs/cairo-lang](https://github.com/starkware-libs/cairo-lang) and are thus licensed under the [Cairo Toolchain License](https://raw.githubusercontent.com/starkware-libs/cairo-lang/master/LICENSE.txt)

