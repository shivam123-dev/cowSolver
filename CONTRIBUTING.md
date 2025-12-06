# Contributing to cowSolver

Thank you for your interest in contributing to the CoW Protocol Solver project!

## Development Workflow

### Branching Strategy

We follow a **Git Flow** branching model:

#### Main Branches
- **`main`** - Production-ready code, always stable
- **`develop`** - Integration branch for features, latest development state

#### Supporting Branches
- **`feature/*`** - New features (branch from `develop`)
- **`bugfix/*`** - Bug fixes (branch from `develop`)
- **`hotfix/*`** - Critical production fixes (branch from `main`)
- **`release/*`** - Release preparation (branch from `develop`)

### Branch Naming Convention

```
feature/solver-engine
feature/bridge-integration
bugfix/order-validation
hotfix/critical-security-fix
release/v0.1.0
```

## Pull Request Process

### 1. Create a Feature Branch

```bash
git checkout develop
git pull origin develop
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Write clean, well-documented code
- Follow Rust best practices and idioms
- Add comprehensive tests
- Update documentation as needed

### 3. Commit Your Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Uniswap V3 routing integration
fix: correct order validation logic
docs: update API documentation
test: add integration tests for settlement
refactor: optimize gas calculation
```

### 4. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Create a PR targeting `develop` branch with:
- Clear description of changes
- Link to related issues
- Test coverage information
- Screenshots/examples if applicable

### 5. Code Review

- Address review comments promptly
- Keep PR scope focused and manageable
- Ensure CI checks pass
- Get at least one approval before merging

### 6. Merge

- Squash commits for clean history
- Delete feature branch after merge
- Update related issues

## Development Setup

### Prerequisites

- Rust 1.70+ 
- Cargo
- Git

### Setup

```bash
# Clone repository
git clone https://github.com/0xtechroot/cowSolver.git
cd cowSolver

# Build project
cargo build

# Run tests
cargo test --workspace

# Run linter
cargo clippy --workspace --all-targets --all-features

# Format code
cargo fmt --all
```

## Code Standards

### Rust Style

- Follow official Rust style guidelines
- Use `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Maximum line length: 100 characters

### Testing

- Write unit tests for all new functions
- Add integration tests for features
- Maintain >80% code coverage
- Test edge cases and error conditions

### Documentation

- Document all public APIs with rustdoc
- Include examples in documentation
- Update README for significant changes
- Add inline comments for complex logic

## Issue Guidelines

### Creating Issues

Use appropriate labels:
- **type:** feature, bug, documentation, testing
- **priority:** critical, high, medium, low
- **area:** core, cross-chain, settlement, amm, infrastructure
- **status:** in-progress, blocked, needs-review

### Issue Template

```markdown
## Description
Clear description of the issue or feature

## Expected Behavior
What should happen

## Current Behavior
What actually happens

## Steps to Reproduce
1. Step one
2. Step two

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
```

## Release Process

### Version Numbering

We use [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

### Release Steps

1. Create release branch from `develop`
2. Update version numbers
3. Update CHANGELOG.md
4. Create PR to `main`
5. After merge, tag release
6. Merge `main` back to `develop`

## Security

Report security vulnerabilities privately to root@ancilar.com

## Questions?

Open a discussion or reach out to maintainers.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
