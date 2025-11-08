---
description: Main guidance for AI assistants working with the typhon repository
author: https://github.com/typhon-dev/typhon
version: 1.0
tags: ["agents", "guidance"]
---
# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Project Overview

Typhon is a statically typed programming language based on Python 3, implemented in Rust with LLVM as the backend.

## Memory Bank Requirements

- **CRITICAL**: **ALL** memory-bank files **MUST** be read at the start of **EACH** task, in **EVERY** mode
- Memory banks contain critical context for understanding project history and current state
- Files in `.roo/memory-bank/` track project context and should be referenced before making changes
- Run `update memory bank` or `UMB` to update all files after significant changes

### Initialization

<thinking>
- **CHECK FOR MEMORY BANK:**
</thinking>

<thinking>
* First, check if the .roo/memory-bank/ directory exists.
</thinking>

<list_files>
<path>.roo</path>
<recursive>false</recursive>
</list_files>

<thinking>
- If .roo/memory-bank/ DOES exist, skip immediately to [If Memory Bank Exists](#if-memory-bank-exists).
</thinking>

### If No Memory Bank

1. **Inform the User:**
    "No Memory Bank was found. I recommend creating one to maintain project context."
2. **Offer Initialization:**
    Ask the user if they would like to initialize the Memory Bank.
3. **Conditional Actions:**
    - If the user declines:
    <thinking>
    I need to proceed with the task without Memory Bank functionality.
    </thinking>

    a. Inform the user that the Memory Bank will not be created.
    b. Set the status to '[MEMORY BANK: INACTIVE]'.
    c. Proceed with the task using the current context if needed or if no task is provided, use the `ask_followup_question` tool.
    - If the user agrees:

    <thinking>
    I need to create the `.roo/memory-bank/` directory and core files. I should use write_to_file for this, and I should do it one file at a time, waiting for confirmation after each. The initial content for each file is defined below. I need to make sure any initial entries include a timestamp in the format YYYY-MM-DD HH:MM:SS.
    </thinking>

4. **Check for `project-brief.md`:**
    - Use list_files to check for `project-brief.md` _before_ offering to create the memory bank.
    - If `project-brief.md` exists:
      - Read its contents _before_ offering to create the memory bank.
    - If no `project-brief.md`:
      - Skip this step (we'll handle prompting for project info _after_ the user agrees to initialize, if they do).

    <thinking>
    I need to add default content for the Memory Bank files.
    </thinking>

    a. Create the `.roo/memory-bank/` directory.
    b. Create `.roo/memory-bank/product-context.md` with `initial_content`.
    c. Create `.roo/memory-bank/active-context.md` with `initial_content`.
    d. Create `.roo/memory-bank/progress.md` with `initial_content`.
    e. Create `.roo/memory-bank/decision-log.md` with `initial_content`.
    f. Create `.roo/memory-bank/system-patterns.md` with `initial_content`.
    g. Set status to '[MEMORY BANK: ACTIVE]' and inform the user that the Memory Bank has been initialized and is now active.
    h. Proceed with the task using the context from the Memory Bank or if no task is provided, use the `ask_followup_question` tool.

### Initial Content

#### Product Context (product-context.md)

```md
# Product Context

This file provides a high-level overview of the project and the expected product that will be created. Initially it is based upon project-brief.md (if provided) and all other available project-related information in the working directory. This file is intended to be updated as the project evolves, and should be used to inform all other modes of the project's goals and context.

*

## Project Goal

*

## Key Features

*

## Overall Architecture

*

YYYY-MM-DD HH:MM:SS - Log of updates made will be appended as footnotes to the end of this file.
```

#### Active Context (active-context.md)

```md
# Active Context

  This file tracks the project's current status, including recent changes, current goals, and open questions.

*

## Current Focus

*

## Recent Changes

*

## Open Questions/Issues

*

YYYY-MM-DD HH:MM:SS - Log of updates made.
```

#### Progress (progress.md)

```md
# Progress

This file tracks the project's progress using a task list format.

*

## Completed Tasks

*

## Current Tasks

*

## Next Steps

*

YYYY-MM-DD HH:MM:SS - Log of updates made.
```

#### Decision Log (decision-log.md)

```md
# Decision Log

This file records architectural and implementation decisions using a list format.

*

## Decision

*

## Rationale

*

## Implementation Details

*

YYYY-MM-DD HH:MM:SS - Log of updates made.
```

#### System Patterns (system-patterns.md)

```md
# System Patterns *Optional*

This file documents recurring patterns and standards used in the project.

It is optional, but recommended to be updated as the project evolves.

*

## Coding Patterns

*

## Architectural Patterns

*

## Testing Patterns

*

YYYY-MM-DD HH:MM:SS - Log of updates made.
```

### If Memory Bank Exists

**READ _ALL_ MEMORY BANK FILES**

<thinking>
I will read all memory bank files, one at a time.
</thinking>

Plan:

1. Read all mandatory files.

   <read_file>
   <args>
     <file>
       <path>.roo/memory-bank/product-context.md</path>
     </file>
     <file>
       <path>.roo/memory-bank/active-context.md</path>
     </file>
     <file>
       <path>.roo/memory-bank/system-patterns.md</path>
     </file>
     <file>
       <path>.roo/memory-bank/decision-log.md</path>
     </file>
     <file>
       <path>.roo/memory-bank/progress.md</path>
     </file>
   </args>
   </read_file>

2. Set status to [MEMORY BANK: ACTIVE] and inform user.
3. Proceed with the task using the context from the Memory Bank or if no task is provided, use the `ask_followup_question` tool.

### General

Begin EVERY response with either '[MEMORY BANK: ACTIVE]' or '[MEMORY BANK: INACTIVE]', according to the current state of the Memory Bank.

### Memory Bank Updates

UPDATE MEMORY BANK THROUGHOUT THE CHAT SESSION, WHEN SIGNIFICANT CHANGES OCCUR IN THE PROJECT.

#### Decision Log Updates

**Trigger**: When a significant architectural decision is made (new component, data flow change, technology choice, etc.). Use your judgment to determine significance.

**Action**:

<thinking>
I need to update decision-log.md with a decision, the rationale, and any implications.
Use insert_content to *append* new information. Never overwrite existing entries. Always include a timestamp.
</thinking>

**Format**: "[YYYY-MM-DD HH:MM:SS] - [Summary of Change/Focus/Issue]"

#### Product Context Updates

**Trigger**: When the high-level project description, goals, features, or overall architecture changes significantly. Use your judgment to determine significance.

**Action**:

<thinking>
A fundamental change has occurred which warrants an update to product-context.md.
Use insert_content to *append* new information or use apply_diff to modify existing entries if necessary. Timestamp and summary of change will be appended as footnotes to the end of the file.
</thinking>

**Format**: "[Optional](YYYY-MM-DD HH:MM:SS) - [Summary of Change]"

#### System Patterns Updates

**Trigger**: When new architectural patterns are introduced or existing ones are modified. Use your judgement.

**Action**:

<thinking>
I need to update system-patterns.md with a brief summary and time stamp.
Use insert_content to *append* new patterns or use apply_diff to modify existing entries if warranted. Always include a timestamp.
</thinking>

**Format**: "[YYYY-MM-DD HH:MM:SS] - [Description of Pattern/Change]"

#### Active Context Updates

**Trigger**: When the current focus of work changes, or when significant progress is made. Use your judgement.

**Action**:

<thinking>
I need to update active-context.md with a brief summary and time stamp.
Use insert_content to *append* to the relevant section (Current Focus, Recent Changes, Open Questions/Issues) or use apply_diff to modify existing entries if warranted.  Always include a timestamp.
</thinking>

**Format**: "[YYYY-MM-DD HH:MM:SS] - [Summary of Change/Focus/Issue]"

#### Progress Updates

**Trigger**: When a task begins, is completed, or if there are any changes Use your judgement.

**Action**:

<thinking>
I need to update progress.md with a brief summary and time stamp.
Use insert_content to *append* the new entry, never overwrite existing entries. Always include a timestamp.
</thinking>

**Format**: "[YYYY-MM-DD HH:MM:SS] - [Summary of Change/Focus/Issue]"

### UMB - Update Memory Bank

**Trigger**: "^(Update Memory Bank|UMB)$"

#### Instructions

- Halt Current Task: Stop current activity
- Acknowledge Command: '[MEMORY BANK: UPDATING]'
- Review Chat History

#### User Acknowledgement Text

"[MEMORY BANK: UPDATING]"

#### Core Update Process

1. Current Session Review:
    - Analyze complete chat history
    - Extract cross-mode information
    - Track mode transitions
    - Map activity relationships
2. Comprehensive Updates:
    - Update from all mode perspectives
    - Preserve context across modes
    - Maintain activity threads
    - Document mode interactions
3. Memory Bank Synchronization:
    - Update all affected *.md files
    - Ensure cross-mode consistency
    - Preserve activity context
    - Document continuation points

#### Task Focus

During a UMB update, focus on capturing any clarifications, questions answered, or context provided _during the chat session_. This information should be added to the appropriate Memory Bank files (likely `active-context.md` or `decision-log.md`), using the other modes' update formats as a guide. _Do not_ attempt to summarize the entire project or perform actions outside the scope of the current chat.

#### Cross-Mode Updates

During a UMB update, ensure that all relevant information from the chat session is captured and added to the Memory Bank. This includes any clarifications, questions answered, or context provided during the chat. Use the other modes' update formats as a guide for adding this information to the appropriate Memory Bank files.

#### Post UMB Actions

- Memory Bank fully synchronized
- All mode contexts preserved
- Session can be safely closed
- Next assistant will have complete context

#### Override Settings

- override_file_restrictions: true
- override_mode_restrictions: true

## Build Commands

- `cargo build` - Build project
- `cargo test --package typhon-compiler` - Run tests for a specific component
- `cargo test -- --nocapture` - Run tests with stdout/stderr output

## Project Structure

```shell
typhon/
└── crates/
    ├── typhon-cli/           # Command-line interface
    ├── typhon-compiler/      # Core compiler components
    │   └── src/
    │       ├── driver/       # Compiler driver
    │       ├── backend/      # LLVM IR generation, code generation
    │       ├── frontend/     # Lexer, parser, AST
    │       ├── middleend/    # AST transformations, optimization
    │       └── typesystem/   # Type checking and inference
    ├── typhon-lsp/           # Language Server Protocol implementation
    ├── typhon-repl/          # Interactive REPL
    ├── typhon-runtime/       # Runtime support
    └── typhon-stdlib/        # Standard library
```

## Code Conventions

- Imports must be grouped by StdExternalCrate (not by default)
- Imports use vertical layout (not mixed)
- Rust 2024 edition style is used
- Doc comments can use special identifiers (CPython, FastAPI, etc.)

## Testing Patterns

- Unit tests in same file as code being tested
- Use property-based testing with `proptest` for edge case discovery
- Snapshot testing with `insta` for complex outputs
- Performance benchmarks with `criterion`
- Mutation testing with `cargo-mutants` for critical components
- Test fixtures stored in `tests/fixtures/`

## Non-obvious Implementation Details

- Hybrid memory management: reference counting, cycle detection, escape analysis
- LSP implementation uses document manager and analyzer engine for incremental analysis
- Type system is central and separates inference from checking
- LLVM pipeline for code generation with custom optimizations
