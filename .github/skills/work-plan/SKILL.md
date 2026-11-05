---
name: work-plan
description: Work on one step of a repository plan in docs/plans/<base-name>/. Use when the user runs `/work-plan <base-name>`, or asks to work on, continue, resume, or review a named plan (for example "work on the comptime-reflection plan" or "continue step 2 of comptime-reflection").
---

# work-plan

Execute exactly one step of a plan stored under `docs/plans/<base-name>/`, following the conventions in `docs/plans/README.md`.

## Input

- **Base name** (required): the plan directory name, for example `comptime-reflection`.
- **Step** (optional): a step number, for example `/work-plan comptime-reflection 2` or "work on step 2 of comptime-reflection".

If no base name is given, list the directories under `docs/plans/` that contain an `index.md`. Use the `ask_user` tool to ask which plan to work on.

## Procedure

1. **Resolve the plan.**
   - Read `docs/plans/<base-name>/index.md`.
   - If it does not exist, list the available plans (directories under `docs/plans/` with an `index.md`) and ask the user which one they meant. Do not guess.
2. **Load context.**
   - Read `docs/plans/README.md` for the conventions: status values, step format, and review gates.
   - Read the plan's `index.md` in full: goals, constraints, the status table, dependencies, and which steps are review gates.
   - Read every supporting file the index lists, such as `findings.md`.
3. **Select the step.**
   - If the user named a step, use it.
   - Otherwise, use the first step in the status table whose status is not **Done** or **Superseded** and whose prerequisites are all **Done**.
   - If the selected step's prerequisites are not all **Done**, report which ones are missing and **stop**.
   - If the step is **Blocked**, report the recorded reason and ask whether the blocker has been resolved before continuing.
   - If every step is Done, say so and stop.
4. **Read the step file** `step-<n>.md` and every file it references as input.
5. **Mark progress.** Set the step's `## Status` and the index status table to **In progress**, unless the step is already In progress or In review.
6. **Execute only that step's tasks.**
   - Work through the `## Tasks` checklist in order unless the step says otherwise.
   - Check each box (`- [x]`) as soon as that task is complete and verified.
   - Append notable decisions or deviations to the step's `## Review log` with the date.
   - Stay within the step's scope. Do not fix unrelated issues. If you find a new problem, record it in the plan (in the step's review log or the plan's findings file) instead of fixing it.
7. **Validate.**
   - Documentation-only steps: verify that every relative Markdown link in created or changed files resolves to an existing file. Check that tables render and that checkbox lists are well-formed.
   - Code steps: run `cargo fmt` (CI enforces `cargo fmt --check`), then `cargo build`, `cargo test`, and `cargo clippy`, plus any focused commands named in the step. Do not report the step complete while any of them fail because of the step's changes.
8. **Agent review (always required).** After validation passes, have a subagent running a **different model** review the step's outputs. Do this before asking the user to review anything and before reporting the step complete. It applies to every step, whether or not the step is a review gate.
   - Use the `task` tool with a review agent: `code-review` for code changes, or `rubber-duck` for plans, designs, and other documentation.
   - Set the `model` parameter explicitly to a model other than the one executing the step. Prefer a different model family.
   - If no different model is available, stop and tell the user. Never review with the same model.
   - Give the reviewer complete context: the plan's goals and constraints, the step's objective, tasks, and acceptance criteria, the relevant findings, and the changed files or diff.
   - Instruct the reviewer to verify claims against the source, not to edit files, and to report each comment with a severity, a location (`path:line`), the problem, and a suggested fix. Severities:
     - **High**: incorrect behavior or claims, a failed acceptance criterion, a violated plan constraint, or a broken build, test, or link.
     - **Medium**: a significant gap, an ambiguity, a missing test, an unverified claim, or an inconsistency between documents.
     - **Low**: style, wording, or minor polish.
   - **Address every High and Medium comment before requesting human review**, in one of these ways:
     - Fix it within the step's scope.
     - If it is out of scope, record it in the plan's findings file or the step's review log instead of fixing it.
     - If it is wrong, record why, with evidence.
     - If resolving it needs a design decision with several reasonable answers, ask the user with `ask_user`.
   - After making fixes, rerun validation (step 7) and request another review from the same reviewer model. Use `write_agent` if the reviewer is still available; otherwise start a new review.
   - Repeat until no High or Medium comment is unresolved. If any remain unresolved after three rounds, stop and report them to the user.
   - Low comments may be fixed or recorded. List any that remain open.
   - Record each round in the step's review log: the date, the reviewer model, the comment count by severity, and how each High and Medium comment was resolved.
9. **Handle review gates.** If the index marks the step as a review gate:
   - When all tasks are complete and agent review (step 8) has no unresolved High or Medium comments, set the status to **In review**.
   - Start with the agent review results: the reviewer model, the High and Medium comments and how they were resolved, and any open Low comments.
   - Present outputs **one document at a time**. For each one, give a short summary, the key design choices, and its `Open questions`.
   - Ask questions **one at a time** with the `ask_user` tool. Offer multiple-choice options when possible, and put the recommended option first, labeled "(Recommended)".
   - Record each answer in the document's `Decisions` section and in the step's review log.
   - Update the document if a decision changes it. Rerun validation (step 7) and agent review (step 8) on the changed parts, then re-present them.
   - Mark a document **Approved** in its `Status` section only after the user explicitly approves it.
   - Mark the step **Done** only after the user explicitly approves the step as a whole. Never infer approval from silence or from approval of a single document.
10. **Close out.**
    - Update the step's `## Status`, the index status table, and the review log.
    - Summarize for the user what changed, the agent review results, the current status, and the **next eligible step**.
    - Do **not** start the next step until the user confirms.

## Rules

- Preserve existing user changes in the working tree. Check `git status` before editing files that are already modified, and never revert them.
- Never create merge commits. Update branches with `git rebase` only.
- Commit only when the user asks.
- Never post comments, PR descriptions, issues, or any other outward communication without first showing the user the exact text and getting approval.
- Follow the plan's constraints in its `index.md`, such as "no code fixes before designs are approved". If a constraint and a task conflict, stop and ask.
- If the plan or step is ambiguous about a design decision with several reasonable answers, ask instead of assuming.
- Keep plan files truthful. Do not check a box, change a status, or record an approval that did not happen.
- Never skip agent review. Never request human review, or report a step complete, while a High or Medium review comment is unresolved.
