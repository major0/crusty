# crustyfmt - Code Formatting Tool

## Introduction

crustyfmt is a code formatting tool for Crusty source files, similar to rustfmt for Rust. It automatically formats Crusty code according to consistent style conventions, ensuring uniform code style across projects.

## Rationale

Consistent code formatting improves readability and reduces cognitive load when reading code. By providing an automated formatting tool, developers can focus on writing code rather than manually formatting it. The tool integrates with development workflows including pre-commit hooks, CI/CD pipelines, and editor plugins.

## Usage

### Basic Usage

```bash
# Format a single file (in-place)
crustyfmt src/main.crst

# Format multiple files
crustyfmt src/main.crst src/lib.crst

# Format all .crst files in a directory recursively
crustyfmt src/
```

### Check Mode

```bash
# Check formatting without modifying files
crustyfmt --check src/main.crst

# Returns exit code 0 if formatted, non-zero if not
```

### Editor Integration

```bash
# Format from stdin to stdout (for editor integration)
crustyfmt --stdin < input.crst > output.crst
```

### Custom Configuration

```bash
# Use custom config file
crustyfmt --config crustyfmt.toml src/
```

## Formatting Rules

crustyfmt applies the following formatting rules:

| Rule | Default |
|------|---------|
| Indentation | 4 spaces per level (no tabs) |
| Line Width | Maximum 100 characters |
| Braces | Opening brace on same line as declaration |
| Spacing | Space after commas, around binary operators |
| Blank Lines | One blank line between top-level items |
| Comments | Preserve position and content of all comments |
| Alignment | Align struct field types and values in initializers |

## Configuration

Create a `crustyfmt.toml` file to customize formatting rules:

```toml
indent_size = 4
max_line_width = 100
space_before_brace = true
space_after_comma = true
```

## Integration

### Pre-commit Hooks

Add to `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: crustyfmt
        name: crustyfmt
        entry: crustyfmt
        language: system
        files: \.crst$
```

### CI/CD

Add to your CI workflow:

```yaml
- name: Check formatting
  run: crustyfmt --check src/
```

### Editor Plugins

crustyfmt supports stdin/stdout mode for editor integration. Configure your editor to run `crustyfmt --stdin` on save.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (formatting applied or files already formatted) |
| 1 | Formatting check failed (files not formatted) |
| 2 | Parse error or invalid input |
| 3 | I/O error or configuration error |

## Properties

crustyfmt guarantees the following properties:

1. **Semantic Preservation**: Formatting preserves the semantic meaning of code. Parsing the original and formatted versions produces semantically equivalent ASTs.

2. **Idempotence**: Formatting is idempotent. Running crustyfmt multiple times produces identical output after the first pass.
