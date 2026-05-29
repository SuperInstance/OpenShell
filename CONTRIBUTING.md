# Contributing to OpenConstruct

Thanks for contributing! This doc covers how to contribute modules, bindings, docs, and code.

## Quick Start

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/my-thing`)
3. Make your changes
4. Run `make test` — all tests must pass
5. Open a PR against `main`

## Ways to Contribute

### Modules
Sense modules live in `crates/` (Rust), `python/` (Python), or as standalone packages. Each module should:
- Follow the module interface: `init()`, `start()`, `stop()`
- Include at least one test
- Document its config options in a README

### Bindings
Language bindings wrap the C ABI (`openconstruct-abi`). When adding a new language:
- Implement the C FFI bindings
- Add tests that exercise every ABI function
- Include install instructions

### Documentation
Docs go in [openconstruct-docs](https://github.com/SuperInstance/openconstruct-docs). For in-repo docs:
- Use clear, concise language
- Include code examples
- Keep the README as the front door — deep docs go in the docs repo

### Bug Reports
- Search existing issues first
- Include: OS, Rust version, steps to reproduce, expected vs actual behavior
- Use the issue template if available

## Code Style

**Rust:**
- `cargo fmt` — no exceptions
- `cargo clippy` — zero warnings
- Edition 2021, MSRV 1.70+

**Python:**
- Python 3.10+
- `black` formatting
- Type hints on all public APIs

**Shell (install.sh):**
- `shellcheck` clean
- `set -euo pipefail`
- Functions over inline code

## Testing

- Every PR must pass `make test`
- New features need tests
- Bug fixes should include a regression test
- Integration tests go in `e2e/`

```bash
make test          # run all tests
cargo test         # Rust tests only
pytest             # Python tests only
```

## PR Process

1. **Small PRs** — one concern per PR
2. **Write a good description** — what, why, how
3. **Link issues** — "Fixes #123" or "Related to #456"
4. **Review** — at least one approval required
5. **CI green** — all checks must pass

## DCO

We use the Developer Certificate of Origin. Sign your commits:

```
git commit -s
```

This adds a `Signed-off-by:` line confirming you have the right to submit the code.

## Questions?

Open an issue or start a discussion on [GitHub Discussions](https://github.com/SuperInstance/OpenConstruct/discussions).
