---
title: Memory Bank
description:
version: 1.0
tags: ["contributing", "agents", "guidance"]
---

This file provides guidance for management of the project memory bank for contributors and AI agents when working with code in this repository.

## Requirements

- **CRITICAL**: **ALL** memory-bank files **MUST** be read at the start of **EACH** task, in **EVERY** mode
- Memory banks contain critical context for understanding project history and current state
- Files in `.memory-bank/` track project context and should be referenced before making changes
- Run `update memory bank` or `UMB` to update all files after significant changes

### Initialization

<thinking>
I need to check if the .memory-bank/ directory and the required files exist.
</thinking>

- Check whether the `.memory-bank/` directory exists at root of the project
- Check whether ALL of these files exist within the `.memory-bank/` directory:
  - `product-context.md`
  - `active-context.md`
  - `progress.md`
  - `decision-log.md`
  - `system-patterns.md`
  - `journal.md`

<thinking>
- If .memory-bank/ DOES NOT exist, proceed to the next section [If No Memory Bank](#if-no-memory-bank).
- If .memory-bank/ DOES exist, skip immediately to [If Memory Bank Exists](#if-memory-bank-exists).
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
    c. Proceed with the task using the current context if needed or if no task is provided, ask for further instructions.
    - If the user agrees:

    <thinking>
    I need to create the `.memory-bank/` directory and core files. I should create one file at a time, waiting for confirmation after each. The initial content for each file is defined below. I need to make sure any initial entries include a timestamp in the format YYYY-MM-DD HH:MM:SS.
    </thinking>

4. **Check for `project-brief.md`:**
    - Check for `project-brief.md` _before_ offering to create the memory bank.
    - If `project-brief.md` exists:
      - Read its contents _before_ offering to create the memory bank.
    - If no `project-brief.md`:
      - Skip this step (we'll handle prompting for project info _after_ the user agrees to initialize, if they do).

    <thinking>
    I need to add default content for the Memory Bank files.
    </thinking>

    a. Create the `.memory-bank/` directory.
    b. Create `.memory-bank/product-context.md` with [initial content](#product-contextmd).
    c. Create `.memory-bank/active-context.md` with [initial content](#active-contextmd).
    d. Create `.memory-bank/progress.md` with [initial content](#progressmd).
    e. Create `.memory-bank/decision-log.md` with [initial content](#decision-logmd).
    f. Create `.memory-bank/system-patterns.md` with [initial content](#system-patternsmd).
    g. Create `.memory-bank/journal.md` with [initial content](#journalmd).
    h. Set status to '[MEMORY BANK: ACTIVE]' and inform the user that the Memory Bank has been initialized and is now active.
    i. Proceed with the task using the context from the Memory Bank or if no task is provided, ask for further instructions.

### Initial Content

#### product-context.md

```md
---
title: Product Context
description: High-level overview of the project
tags: ["memory-bank", "documentation", "product-context", "project-overview", "product", "architecture", "features"]
---

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

#### active-context.md

```md
---
title: Active Context
description: Current status of the project including recent changes and goals
tags: ["memory-bank", "documentation", "active-context", "current-status", "active", "status", "changes", "questions"]
---

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

#### progress.md

```markdown
---
title: Progress
description: Tracks the project's progress using a task list format
tags: ["memory-bank", "documentation", "progress", "tasks", "tracking", "implementation"]
---

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

#### decision-log.md

```markdown
---
title: Decision Log
description: Records architectural and implementation decisions in the project
tags: ["memory-bank", "documentation", "decision-log", "architecture", "implementation", "decisions", "design"]
---
<!-- markdownlint-disable-file no-duplicate-heading -->

This file records architectural and implementation decisions using a list format.

---

## Decision

*

## Rationale

*

## Implementation Details

*

---

YYYY-MM-DD HH:MM:SS - Log of updates made.
```

#### system-patterns.md

```markdown
---
title: System Patterns *Optional*
description: Documents recurring patterns and standards used in the project
tags: ["memory-bank", "documentation", "system-patterns", "coding-patterns", "architectural-patterns", "testing-patterns", "patterns", "standards", "code-organization", "architecture", "testing"]
---

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

#### journal.md

```markdown
---
title: Development Journal
description: A chronological record of changes, decisions, challenges, and solutions during development
tags: ["memory-bank", "documentation", "journal", "changes", "decisions", "challenges", "solutions"]
---

This file provides a chronological narrative of the development process, documenting significant technical challenges, solutions, and architectural decisions as they occur.

## YYYY-MM-DD: [Title of Entry]

### [Section 1: Challenge/Task Description]

*

### [Section 2: Solution Approach]

*

### [Section 3: Implementation Details]

*

### [Section 4: Lessons Learned]

*

YYYY-MM-DD HH:MM:SS - Initial creation of development journal.
```

### If Memory Bank Exists

**READ _ALL_ MEMORY BANK FILES**

<thinking>
I will read all memory bank files.
</thinking>

Plan:

1. Read all the memory bank files to understand the project context:
   - .memory-bank/product-context.md
   - .memory-bank/active-context.md
   - .memory-bank/system-patterns.md
   - .memory-bank/decision-log.md
   - .memory-bank/progress.md
   - .memory-bank/journal.md

2. Set status to [MEMORY BANK: ACTIVE] and inform user.
3. Proceed with the task using the context from the Memory Bank or if no task is provided, ask for further instructions.

### General

Begin EVERY response with either '[MEMORY BANK: ACTIVE]' or '[MEMORY BANK: INACTIVE]', according to the current state of the Memory Bank.

### Memory Bank Updates

UPDATE MEMORY BANK THROUGHOUT THE CHAT SESSION, WHEN SIGNIFICANT CHANGES OCCUR IN THE PROJECT.

#### Decision Log Updates

**Trigger**: When a significant architectural decision is made (new component, data flow change, technology choice, etc.). Use your judgment to determine significance.

**Action**:

<thinking>
I need to update decision-log.md with a decision, the rationale, and any implications.
Add new information at the appropriate location. Never overwrite existing entries. Always include a timestamp.
All timestamps should be added at the BOTTOM of the file. Maintaining chronological order of timestamps is **CRITICAL**.
</thinking>

**Format**: "[YYYY-MM-DD HH:MM:SS] - [Summary of Change/Focus/Issue]"

#### Product Context Updates

**Trigger**: When the high-level project description, goals, features, or overall architecture changes significantly. Use your judgment to determine significance.

**Action**:

<thinking>
A fundamental change has occurred which warrants an update to product-context.md.
Add new information or modify existing entries if necessary. Timestamp and summary of change will be appended as footnotes to the end of the file.
</thinking>

**Format**: "[Optional](YYYY-MM-DD HH:MM:SS) - [Summary of Change]"

#### System Patterns Updates

**Trigger**: When new architectural patterns are introduced or existing ones are modified. Use your judgement.

**Action**:

<thinking>
I need to update system-patterns.md with a brief summary and time stamp.
Add new patterns or modify existing entries if warranted. Always include a timestamp.
</thinking>

**Format**: "[YYYY-MM-DD HH:MM:SS] - [Description of Pattern/Change]"

#### Active Context Updates

**Trigger**: When the current focus of work changes, or when significant progress is made. Use your judgement.

**Action**:

<thinking>
I need to update active-context.md with a brief summary and time stamp.
Add to the relevant section (Current Focus, Recent Changes, Open Questions/Issues) or modify existing entries if warranted. Always include a timestamp.
</thinking>

**Format**: "[YYYY-MM-DD HH:MM:SS] - [Summary of Change/Focus/Issue]"

#### Progress Updates

**Trigger**: When a task begins, is completed, or if there are any changes Use your judgement.

**Action**:

<thinking>
I need to update progress.md with a brief summary and time stamp.
Add the new entry, never overwrite existing entries. Always include a timestamp.
</thinking>

**Format**: "[YYYY-MM-DD HH:MM:SS] - [Summary of Change/Focus/Issue]"

#### Journal Updates

**Trigger**: After significant technical challenges are solved, architectural changes are implemented, or important development milestones are reached.

**Action**:

<thinking>
I need to update journal.md with a detailed narrative of the technical challenge, solution approach, implementation details, and lessons learned.
Add new journal entries in chronological order (oldest to newest). Never overwrite existing entries. Always include a timestamp.
</thinking>

**Format**: "## YYYY-MM-DD: [Title of Entry]" with subsections for Challenge, Solution, Implementation, and Lessons Learned.

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
