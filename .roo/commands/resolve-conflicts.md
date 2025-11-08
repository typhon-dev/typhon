---
description: Systematic workflow for resolving git merge conflicts with special handling for memory bank files
argument-hint: <view | memory | config | code | test | doc>
---
# Git Conflict Resolution Workflow

## Usage

```shell
/resolve-conflicts <view | memory | config | code | test | doc>
```

## Description

This command provides a systematic approach for resolving git merge conflicts with special focus on maintaining context and preserving critical information, especially for memory bank files.

## Options

- `view`: List all conflicted files and categorize them by type
- `memory`: Resolve conflicts in memory bank files (`.roo/memory-bank/*.md`)
- `config`: Resolve conflicts in configuration files (`.json`, `.yml`, etc.)
- `code`: Resolve conflicts in source code files (TypeScript, JavaScript, etc.)
- `test`: Resolve conflicts in test files
- `doc`: Resolve conflicts in documentation files

## Examples

```shell
/resolve-conflicts view
/resolve-conflicts memory
```

## Workflow Overview

1. Identify conflicts and classify by file type
2. Resolve memory bank files with special attention to chronological order
3. Resolve configuration files maintaining consistency and validity
4. Resolve source code files preserving functionality and intent
5. Resolve test files and documentation
6. Verify resolution and complete the merge

## Important Rules

- Do NOT commit or push during conflict resolution
- For `package.json`, keep dependencies ordered alphabetically
- For `raw-reflection-log.md`, keep entries in chronological order with most recent at the top
- Memory bank files require special handling to preserve knowledge

## Key Strategies By File Type

### Memory Bank Files (`.roo/memory-bank/*.md`)

- Use incoming changes as the foundation
- Preserve chronological ordering of entries
- Remove duplicate content
- Ensure all meaningful information is retained

### Configuration Files (`.json`, `.yml`, etc.)

- Maintain consistency and validity
- Preserve essential settings
- Validate after merging
- For `package.json`, ensure dependencies remain alphabetically ordered

### Source Code Files

- Preserve functionality and intent
- Understand both changes
- Maintain consistent coding style
- Ensure types and interfaces remain compatible

## Resolution Steps

1. Back up your current state with `git stash save`
2. Understand the context by examining branch differences
3. List and categorize all conflicted files
4. Follow specific resolution strategies by file type
5. Verify all resolved files for content preservation
6. Run tests and validation to ensure functionality
7. Stage resolved files when complete
