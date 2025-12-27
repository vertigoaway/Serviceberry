# Contributing to Serviceberry

Thanks for your interest in contributing! We welcome bug fixes, new features, documentation improvements, and testing.

## Setup

1. Install [Rust](https://rust-lang.org/tools/install/) and system dependencies (see README)
2. Fork and clone the repo
3. Enable required services:
   ```bash
   sudo systemctl enable --now bluetooth avahi-daemon
   ```
4. Build and test:
   ```bash
   cargo build
   cargo test
   ```

## Code Guidelines

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes
- Add tests for new functionality
- Document public APIs with doc comments

## Pull Requests

Include in your PR:
- Clear description of changes
- Reference to related issues
- Test coverage for new code

## Reporting Issues

For bugs, include:
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version)
- Relevant error logs

For features, describe the problem it solves and your proposed solution.

---

Questions? Open an issue or check existing ones. Thanks for contributing! 🎉
