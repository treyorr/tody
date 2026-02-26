# Contributing to tody

Thanks for your interest in contributing! Here's how to get started.

## Setup

1. **Clone the repo**

   ```sh
   git clone https://github.com/treyorr/tody.git
   cd tody
   ```

2. **Install the toolchain**

   If you use [mise](https://mise.jdx.dev/):

   ```sh
   mise install
   ```

   Otherwise, install [Rust](https://rustup.rs/) 1.85+ with the `clippy` and `rustfmt` components.

3. **Run the CI checks locally**

   ```sh
   mise run ci        # or run each step manually:
   cargo fmt --all -- --check
   cargo check --all-targets
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-targets
   ```

## Making changes

1. Fork the repo and create a branch from `main`.
2. Make your changes — keep commits small and focused.
3. Add or update tests for any new behaviour.
4. Run `mise run ci` (or the individual commands above) and make sure everything passes.
5. Open a pull request with a clear description of **what** and **why**.

## Code style

- Run `cargo fmt` before committing.
- All clippy warnings are treated as errors (`-D warnings`).
- Keep functions small. Prefer clear names over comments.

## Reporting bugs

Open an [issue](https://github.com/treyorr/tody/issues) with:

- Your OS and architecture
- tody version (`tody --version`)
- Steps to reproduce
- Expected vs actual behaviour

## Feature requests

Open an issue describing the use case. Keep it focused — tody aims to stay tiny and tidy.
