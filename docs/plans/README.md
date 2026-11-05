# Plans

This directory holds implementation plans. Each plan lives in its own directory and is identified by its **base name**, which is the directory name. For example, `comptime-reflection` refers to `docs/plans/comptime-reflection/`.

To work on a plan with GitHub Copilot, use the repository skill [`work-plan`](../../.github/skills/work-plan/SKILL.md):

```text
/work-plan <base-name>
```

The skill reads the plan, picks the next eligible step, and runs only that step.

## Plan index

| Plan | Status | Summary |
|---|---|---|
| [comptime-reflection](./comptime-reflection/index.md) | **Active** | Redirect Patina toward compile-time (`comptime`) metaprogramming with static reflection, incremental compilation, and incremental reflection. Begins with design work and guided review. |

The superseded phase-separation plan has been removed. Its assessment and unfinished work remain in the [comptime-reflection carry-over table](./comptime-reflection/index.md#phase-separation-carry-over).

## Layout

```text
docs/plans/<base-name>/
├── index.md        # required: goals, constraints, status table, dependencies
├── step-1.md       # one file per step, numbered from 1
├── step-2.md
├── ...
└── <supporting>.md # optional: findings, audits, inventories referenced by steps
```

- `index.md` is the entry point. It states the plan's goals, constraints, the **status table** for all steps, and step dependencies.
- `step-<n>.md` files are numbered in intended execution order. A plan may intentionally leave later steps unwritten. When it does, an earlier step's job is to create them, and the index says so.
- Supporting files hold shared reference data, such as `findings.md`. Steps cite them by stable IDs rather than duplicating them.

## Status values

The index status table and each step's `## Status` section use exactly one of the following values:

| Status | Meaning |
|---|---|
| Not started | No work has begun. |
| In progress | Work is underway. Some tasks may be checked. |
| In review | All tasks are complete and agent review has no unresolved High or Medium comments. The step is a review gate awaiting explicit user approval. |
| Done | Acceptance criteria are met. For review gates, the user has explicitly approved. |
| Blocked | A prerequisite or external decision is missing. The reason is recorded in the step. |
| Superseded | Replaced by another plan or step, which is linked. |

## Step file format

Every step file uses these sections, in this order:

1. `# Step <n>: <title>`
2. `## Objective`: one or two paragraphs describing the outcome.
3. `## Prerequisites`: the steps that must be **Done** first, plus any required inputs.
4. `## Tasks`: Markdown checkboxes (`- [ ]`) that are small enough to verify individually. Checkboxes are updated as work progresses.
5. `## Acceptance criteria`: observable conditions that make the step Done.
6. `## Status`: one status value from the table above, optionally followed by a short note.
7. `## Review log`: a dated, append-only list of decisions, approvals, and notable changes made while executing the step.

### Agent review

Every step is reviewed by a subagent running a **different model** from the one that executed it. This happens after validation and before any human review, and before the step is reported complete. The reviewer reports each comment as **High**, **Medium**, or **Low**.

Every High and Medium comment must be resolved before human review is requested. A comment is resolved by fixing it in scope, by recording it in the plan when it is out of scope, or by recording evidence that it is wrong. After fixes, the same reviewer model reviews again.

Each review round is recorded in the step's review log. The [`work-plan`](../../.github/skills/work-plan/SKILL.md) skill defines the full procedure.

### Review gates

A step marked as a **review gate** in the index cannot be marked Done by the agent alone. When its tasks are complete and [agent review](#agent-review) has no unresolved High or Medium comments, it moves to **In review**. The agent then walks the user through the outputs one document at a time and records each decision. The step becomes Done only after the user explicitly approves.

## Conventions

- Plans change documentation and code only through their steps. Do not fix out-of-scope issues while executing a step. Record them in the plan instead.
- Steps cite findings, designs, and files by stable ID or path, with line numbers where useful.
- When a plan supersedes another, mark the old plan **Superseded** in its index and in this README, and link to the replacement.
- When superseded documentation is removed, preserve relevant findings and unfinished work in the replacement plan, and update links and prerequisites so deleted files are no longer required inputs.
- Never create merge commits; update branches with `git rebase`.
