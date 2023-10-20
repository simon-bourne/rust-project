# Rust Workflow Utils

[![tests](https://github.com/simon-bourne/rust-project/actions/workflows/tests.yml/badge.svg)](https://github.com/simon-bourne/rust-project/actions/workflows/tests.yml)

Utilities for creating [cargo-xtask](https://github.com/matklad/cargo-xtask) projects. Create an `xtask` crate with a `main.rs` something like:

```rust
use xtask_base::{build_readme, ci::CI, generate_open_source_files, CommonCmds, WorkflowResult};

fn main() {
    CommonCmds::run(|| ci().run(), code_gen)
}

fn code_gen(check: bool) -> WorkflowResult<()> {
    build_readme(".", check)?;
    generate_open_source_files(2022, check)?;
    github_actions(check)
}

fn github_actions(check: bool) -> WorkflowResult<()> {
    ci().write(check)
}

fn ci() -> CI {
    CI::standard_workflow()
}

```
