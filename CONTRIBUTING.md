# Contributing to Clash

Thank you for your interest in contributing to Clash! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.93.0 or later
- Git 2.30+
- cargo-watch (optional, for auto-rebuild)

### Setup

```bash
# Clone the repository
git clone https://github.com/clash-sh/clash.git
cd clash

# Install development tools
rustup component add clippy rustfmt
cargo install cargo-watch cargo-insta

# Build and test
cargo build
cargo test
```

### Development Workflow

```bash
# Auto-rebuild on changes
cargo watch -x check -x test

# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Run tests
cargo test
```

## Code Style

We follow standard Rust conventions with these specifics:

- **Always run** `cargo fmt` and `cargo clippy` before submitting PR

## Project Principles

1. **Read-only**: Never modify git state

## Making Changes

### Before You Start

1. Check existing [issues](https://github.com/clash-sh/clash/issues)
2. For anything beyond a small fix, open an issue first to discuss
4. Review [CLAUDE.md](CLAUDE.md) for project conventions

### Keep PRs Focused

Each PR should address one thing — a bug fix, a feature, or a refactor, not all three. If you find unrelated issues while working, open a separate issue or PR.

### Commit Messages

Follow conventional commits:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation only
- `test:` Test additions/changes
- `refactor:` Code restructuring
- `perf:` Performance improvements
- `chore:` Maintenance tasks

### Pull Request Process

1. Fork and create a feature branch from `main`
2. Make your changes following the guidelines above
3. Add/update tests as needed
4. Update documentation if applicable
5. Ensure CI passes (fmt, clippy, tests)
6. Submit PR against `main` with a clear description
7. PRs are squash-merged to keep history clean

### Key Areas We Need Help

- **MCP server implementation** — for AI agent integration
- **Platform testing** — macOS, Linux, Windows, different git versions
- **Conflict detection edge cases** — renames, submodules, binary files
- **Documentation** — examples, tutorials, use cases
- **Package managers** — Homebrew formula, apt/yum packages

Raise an issue if you think something else would be interesting! 

## Getting Help

- Check [CLAUDE.md](CLAUDE.md) for detailed guidelines
- Open an [issue](https://github.com/clash-sh/clash/issues) for questions
- Discussions in [GitHub Discussions](https://github.com/clash-sh/clash/discussions)

## Recognition

Contributors will be recognized in:

- [AUTHORS](AUTHORS) file
- Release notes
- Project documentation

## License

By contributing, you agree that your contributions will be licensed under the MIT License.