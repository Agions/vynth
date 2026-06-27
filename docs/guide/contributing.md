# Contributing

Contributions are welcome. Here's how to get started.

## Quick Links

- [Code of Conduct](https://github.com/Agions/synerix/blob/main/CODE_OF_CONDUCT.md)
- [Issue Tracker](https://github.com/Agions/synerix/issues)

## Development Setup

### Prerequisites

- Rust 1.75+
- Node.js 18+ (for docs)
- Git

### Clone and Build

```bash
git clone https://github.com/Agions/synerix.git
cd synerix

# Build Rust binary
cargo build

# Run tests
cargo test

# Build docs (optional)
cd docs && npm install && npm run docs:dev
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy --all-targets --all-features`
- Write tests for new functionality
- Keep functions small and focused

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add slash menu widget
fix: correct token formatting edge case
docs: update getting started guide
style: align rustfmt settings
refactor: extract sandbox classifier
test: add integration tests for MCP
chore: bump ratatui to 0.29
```

## Pull Requests

1. Fork the repo
2. Create a branch: `git checkout -b feat/your-feature`
3. Make changes and commit
4. Push: `git push origin feat/your-feature`
5. Open a PR on GitHub

## Questions?

Open a [Discussion](https://github.com/Agions/synerix/discussions) or join the community channels linked in our README.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](https://github.com/Agions/synerix/blob/main/LICENSE).
