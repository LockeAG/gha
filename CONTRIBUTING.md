# Contributing to gha

Thanks for your interest. Here's how to get involved.

## Reporting bugs

Open an [issue](https://github.com/LockeAG/gha/issues/new?template=bug_report.md). Include:
- What you expected vs what happened
- Terminal emulator and OS
- `gha --help` output (confirms version)
- Steps to reproduce

## Requesting features

Open an [issue](https://github.com/LockeAG/gha/issues/new?template=feature_request.md). Describe the problem you're trying to solve, not just the solution you want. Context helps.

## Pull requests

1. Fork the repo
2. Create a branch from `main`
3. Make your changes
4. `cargo build` — must compile with zero warnings
5. Test manually against a real GitHub org/repo
6. Open a PR with a clear description of what and why

### Development setup

```sh
git clone https://github.com/YOUR_USERNAME/gha.git
cd gha
cargo build
./target/debug/gha --org YourOrg
```

Requires Rust 1.74+ and a GitHub token (`gh auth login` is the easiest path).

### Code style

- Follow existing patterns — read the code before changing it
- No unnecessary dependencies
- No `unsafe`
- Minimize allocations in the render loop
- Keep commits atomic and messages in imperative mood

### What makes a good PR

- Solves one problem
- Doesn't break existing functionality
- Includes context on why, not just what
- Follows the existing code style

### What we won't merge

- Cosmetic-only refactors with no functional benefit
- Features without a clear use case
- PRs that add heavy dependencies for marginal gains

## Architecture overview

Read the Architecture section in README.md. Key points:

- Three async tasks → one mpsc channel → main thread renders
- No shared mutexes, unidirectional data flow
- Theme is a global `OnceLock<Theme>`, initialized once at startup
- Config layering: CLI flags > config.toml > defaults

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
