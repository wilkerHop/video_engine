# Contributing to Video Engine

Thank you for your interest in contributing to the Video Engine project! This document provides guidelines for testing, commits, and the overall development workflow.

## Testing Standards

> **CRITICAL RULE**: Every file that exports a function MUST have a corresponding test module.

This rule is strictly enforced by our health check pipeline.

###Before Submitting Changes

1. **Write Tests**: Add unit tests for all new functions
2. **Run Tests**: Execute `cargo test` - all tests must pass
3. **Health Check**: Run `./scripts/check_test_coverage.sh` - must pass
4. **Fix Warnings**: Ensure no compiler warnings (`cargo clippy`)

### Test Coverage Requirements

- ✅ Unit tests for all public functions
- ✅ Edge case testing (invalid inputs, boundary conditions)
- ✅ Error condition handling
- ✅ Integration tests for main flows

### Writing Good Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange: Set up test data
        let input = create_test_input();
        
        // Act: Call the function
        let result = function_under_test(input);
        
        // Assert: Verify the result
        assert_eq!(result, expected_value);
    }
    
    #[test]
    fn test_error_handling() {
        let result = function_under_test(invalid_input);
        assert!(result.is_err());
    }
}
```

---

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/) for all commit messages.

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature or functionality
- `fix`: Bug fix
- `docs`: Documentation changes only
- `test`: Adding or updating tests
- `refactor`: Code changes that neither fix bugs nor add features
- `chore`: Build process, dependencies, or auxiliary tools
- `ci`: CI/CD configuration changes
- `perf`: Performance improvements

### Scopes

- `script`: Changes to script.rs (data structures)
- `parser`: Changes to parser.rs (JSON parsing)
- `assets`: Changes to assets.rs (asset loading)
- `deps`: Dependency updates
- `ci`: CI/CD pipeline
- `tests`: Test-related changes
- `docs`: Documentation

### Examples

**Good Commits:**
```bash
feat(parser): add TOML script parsing support
fix(assets): correct path resolution for Windows
test(script): add comprehensive tests for default functions
docs(readme): update testing guidelines
chore(deps): upgrade serde to 1.0.200
refactor(assets): simplify asset caching logic
```

**Bad Commits:**
```bash
❌ "Added feature"           # Missing type and scope
❌ "Fix bug"                 # Too vague
❌ "feat(parser): Added."    # Capitalized, has period
❌ "Updated code"            # No type, vague
```

### Commit Message Rules

1. Use imperative mood: "add" not "added" or "adds"
2. Don't capitalize the first letter of description
3. No period at the end of description
4. Keep description under 72 characters
5. Provide context in body for complex changes
6. Reference issues in footer: "Closes #123"

---

## Development Workflow

### 1. Create Feature Branch

```bash
git checkout -b feat/your-feature-name
# or
git checkout -b fix/bug-description
```

### 2. Make Changes

- Write code following Rust best practices
- Add tests for all new functions
- Keep commits atomic and focused
- Write good commit messages

### 3. Test Locally

```bash
# Run all tests
cargo test

# Run specific module
cargo test script

# Run with coverage
cargo test -- --nocapture

# Health check
./scripts/check_test_coverage.sh

# Check for warnings
cargo clippy
```

### 4. Commit Changes

```bash
# Stage changes
git add src/script.rs

# Commit with proper format
git commit -m "feat(script): add video layer support"

# If commit message is too long, use editor
git commit
# Then write multi-line message in editor
```

### 5. Push and Create PR

```bash
git push origin feat/your-feature-name
```

Then create a Pull Request on GitHub.

---

## Code Review Checklist

Before requesting review, ensure:

- [ ] All tests pass (`cargo test`)
- [ ] Health check passes (`./scripts/check_test_coverage.sh`)
- [ ] No compiler warnings (`cargo clippy`)
- [ ] New functions have unit tests
- [ ] Commits follow conventional format
- [ ] Documentation updated (if needed)
- [ ] Code follows project style

---

## Running the Health Check

The health check enforces our testing rule:

```bash
./scripts/check_test_coverage.sh
```

**What it checks:**
1. Every file in `src/` with functions has a `#[cfg(test)]` module
2. All unit tests pass

**Expected output:**
```
🔍 Checking test coverage compliance...
✅ PASS: src/script.rs has test module
✅ PASS: src/parser.rs has test module
✅ PASS: src/assets.rs has test module

🧪 Running all unit tests...
<test results>

✅ Health check PASSED: All files with functions have tests
```

---

## Project Structure

```
video_engine/
├── src/
│   ├── lib.rs          # Library exports
│   ├── main.rs         # Demo application
│   ├── script.rs       # Data structures + tests
│   ├── parser.rs       # JSON parsing + tests
│   └── assets.rs       # Asset loading + tests
├── examples/
│   ├── simple.json     # Example video script
│   └── assets/         # Example assets
├── scripts/
│   └── check_test_coverage.sh  # Health check script
├── tests/              # Integration tests
├── Cargo.toml          # Dependencies
└── README.md           # Project documentation
```

---

## Getting Help

- **Questions**: Open an issue with the `question` label
- **Bugs**: Open an issue with the `bug` label + reproduction steps
- **Features**: Open an issue with the `enhancement` label

---

## License

MIT - See LICENSE file for details.

---

**Remember**: Quality over quantity. Well-tested, properly committed code makes everyone's life easier! 🚀
