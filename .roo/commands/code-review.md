---
description: Perform comprehensive code reviews on GitHub pull requests or local code changes
argument-hint: <github#<pull request number> | local>
---
# Comprehensive Code Review

## Usage

```shell
/code-review <github#<pull request number> | local>
```

## Description

This command guides you through reviewing code changes, either through GitHub pull requests or local code comparisons, ensuring code quality and adherence to project standards.

## Options

- `github#<pull request number>`: Review a GitHub pull request (requires MR number)
- `local`: Review local code changes compared to `origin/dev` branch

## Examples

```shell
/code-review github#42
/code-review local
```

## Workflow Steps

### For GitHub pull Request Reviews

1. Retrieves pull request information using Github MCP server tools
2. Examines changed files and their diffs
3. Analyzes code against style guides and best practices
4. Provides detailed feedback through comments on the MR

### For Local Code Reviews

1. Identifies changed files compared to the `origin/dev` branch
2. Analyzes local changes against style guides and best practices
3. Provides a comprehensive review report

## Review Checklist

- Style guide compliance (TypeScript, Markdown, YAML)
- Best practices verification (DRY, SRP, error handling)
- Memory bank updates verification
- Unused code detection
- Consistency with existing codebase

## Implementation Notes

This command helps ensure code quality by providing a structured review process that checks:

1. Style Guide Compliance
   - TypeScript: quotes, semicolons, indentation, naming conventions
   - Markdown: heading hierarchy, spacing, code blocks
   - YAML: document markers, spacing, quotation, key organization

2. Best Practices
   - Code duplication
   - Function responsibility
   - Error handling
   - TypeScript typing
   - Documentation
   - Design patterns

3. Code Organization
   - File structure consistency
   - Interface extensions
   - Error handling conventions
   - Naming consistency
   - Documentation style
