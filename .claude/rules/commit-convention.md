# Commit Conventions

surge-tui uses [Conventional Commits](https://www.conventionalcommits.org/) format.

## Core Principles

**⚠️ Only create commits when explicitly requested by the user**

- ❌ Don't proactively suggest "Should I commit this"
- ❌ Don't auto-commit after completing a feature
- ❌ Don't assume the user wants to commit code

**Only when the user explicitly says:**
- "Commit the code"
- "Create a commit"
- "Commit these changes"
- Or uses the `/commit` command

**Should you execute the commit operation.**

Users might be experimenting, debugging, or planning multiple changes before committing. Don't make decisions for users.

## Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type (Required)

**Main types:**
- `feat` - New feature
- `fix` - Bug fix
- `refactor` - Refactoring (no functional change)
- `perf` - Performance optimization
- `style` - Code formatting (no logic change)
- `test` - Add or modify tests
- `docs` - Documentation changes
- `chore` - Build, dependencies, tool configuration

**Examples:**
```
feat: add policy latency testing
fix: fix nested policy group parsing error
refactor: refactor HTTP client fallback logic
perf: optimize policy list rendering performance
docs: update i18n documentation
```

### Scope (Optional)

**Indicates affected area:**
- `domain` - Domain layer
- `infra` - Infrastructure layer
- `app` - Application layer
- `ui` - UI layer
- `i18n` - Internationalization
- `config` - Configuration

**Examples:**
```
feat(ui): add notification history popup
fix(infra): fix CLI client timeout handling
refactor(domain): simplify Policy data model
```

**Small projects can omit scope:**
```
feat: add latency testing
fix: fix crash issue
```

### Subject (Required)

**Concise description (< 50 characters):**
- Use imperative mood: "add" not "added"
- Lowercase first letter (not applicable for Chinese)
- No period at the end
- Explain "what", not "how"

**Good examples:**
```
feat: add non-blocking latency testing
fix: fix nested policy group display
refactor: extract common HTTP error handling
```

**Bad examples:**
```
feat: added a new feature for testing policy latency using tokio::spawn for concurrent execution
fix: Bug fix.
update: changed some things
```

### Body (Optional)

**Detailed explanation (if needed):**
- Explain "why", not "what"
- State context, motivation, impact
- Each line < 72 characters

**Example:**
```
feat: implement HTTP API fallback to CLI

HTTP API might be unavailable due to configuration errors,
fallback to surge-cli ensures basic functionality.

Fallback order:
1. HTTP API - most complete
2. surge-cli - basic functionality
3. system - only check process status
```

### Footer (Optional)

**Breaking changes:**
```
BREAKING CHANGE: API Key configuration changed from environment variable to config file

Migration method:
export SURGE_API_KEY -> http_api_key in config.toml
```

**Link issues:**
```
Closes #123
Fixes #456
Refs #789
```

## Real-world Examples

### New Feature
```
feat: complete i18n Phase 2 - UI integration

All UI components migrated to i18n::Translate trait.
Supports compile-time switching between Chinese/English versions.

Default Chinese: cargo build
English version: cargo build --no-default-features --features lang-en-us

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

### Bug Fix
```
fix(ui): fix latency display error in nested policy groups

Nested policy groups (e.g., Auto Group → Select Group → Proxy) weren't
correctly recursively finding the final policy, causing empty latency display.

Fix: add recursive lookup logic in resolve_policy_chain().
```

### Refactoring
```
refactor(infra): unify HTTP client error handling

Extract common handle_http_error() method, reduce code duplication.
All HTTP calls use unified timeout, retry, error conversion logic.

No external interface changes, pure internal refactoring.
```

### Performance Optimization
```
perf(ui): optimize policy list rendering

Use StatefulWidget to cache rendering state, avoid recalculating every frame.
Test results: CPU usage decreased from 15% to 3%.
```

### Documentation
```
docs: add .claude/rules documentation

Added four rule documents:
- project-structure.md - Project structure
- i18n.md - Internationalization guidelines
- code-style.md - Code guidelines
- commit-convention.md - Commit conventions
```

## Special Cases

### Multiple File Modifications

**Group by logic, split into multiple commits:**
```bash
# ✅ Good
git commit -m "feat(domain): add Policy latency field"
git commit -m "feat(infra): implement latency testing API"
git commit -m "feat(ui): display policy latency"

# ❌ Bad
git commit -m "add latency testing feature, modified domain, infra, ui and multiple files"
```

**Exception:** When cross-layer modifications are highly coupled logically, can use single commit.

### WIP Commits

**Use WIP during development:**
```
WIP: implement latency testing (incomplete)
```

**Rebase before merging to combine into meaningful commits:**
```bash
git rebase -i HEAD~5
# Squash WIP commits
```

### Co-authored

**Collaboration with Claude or others:**
```
feat: implement non-blocking latency testing

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

## Commit Checklist

**Check before committing:**
- [ ] Type correct (feat/fix/refactor/docs/...)
- [ ] Subject < 50 characters, clear description
- [ ] Add Body to explain "why" if needed
- [ ] Run `cargo fmt` and `cargo clippy`
- [ ] Tests pass (if tests exist)
- [ ] Feature complete (don't commit half-finished work)

## Tools

**commitlint (optional):**
```bash
# Install
npm install -g @commitlint/cli @commitlint/config-conventional

# Configure .commitlintrc.json
{
  "extends": ["@commitlint/config-conventional"]
}

# Git hook
npx husky add .husky/commit-msg 'npx commitlint --edit $1'
```

**But for small projects, manual compliance is enough.** Don't use tools for the sake of tools.

## Anti-patterns

**Don't:**
- ❌ `update code` - doesn't explain what was done
- ❌ `fix bug` - doesn't explain what was fixed
- ❌ `WIP` - don't commit to main
- ❌ `fix some issues, update docs, refactor code` - should split into multiple commits
- ❌ `feat: add new feature` - too vague, specify what feature

---

*"Good commit messages serve as a communication device between team members."*
