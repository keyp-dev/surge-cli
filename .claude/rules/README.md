# Claude Rules - surge-tui Project Guidelines

These rule documents are automatically loaded into Claude Code's context to guide code generation and modifications.

## Rule Documents

1. **[project-structure.md](project-structure.md)** - Project structure and layering rules
   - Clean Architecture layering
   - Dependency direction rules
   - Guide for adding new features

2. **[i18n.md](i18n.md)** - Internationalization guidelines
   - Compile-time language selection
   - Translate trait usage
   - Adding new language workflow

3. **[code-style.md](code-style.md)** - Code style guidelines
   - SOLID principles application
   - Rust best practices
   - Error handling, comments, testing

4. **[commit-convention.md](commit-convention.md)** - Commit conventions
   - Conventional Commits format
   - Type, Scope, Subject rules
   - Real-world examples

## Auto-loading

These rules are automatically loaded in the following situations:

- **Global loading** - README.md (this file)
- **By file type** - When editing related files
  - `.rs` files → code-style.md
  - `i18n/` directory → i18n.md
  - Any Rust file → project-structure.md
  - Git commit → commit-convention.md

See [Claude Code documentation](https://code.claude.com/docs/en/memory#modular-rules-with-claude%2Frules%2F).

## Updating Rules

After modifying rule documents, Claude Code will automatically load the new version in the next session. No restart needed.

---

*"The only way to go fast is to go well."* — Robert C. Martin
