# Aero Agent Rules

These rules apply to every agent working in this repository. The repository's
default branch is `master`; integration work belongs on
`agent/aero-integration` or another explicitly assigned branch/worktree.

## Core constraints

- Never commit directly to `master`, force-push, rewrite published history, or
  publish a release, package, registry record, benchmark claim, or external
  artifact.
- Do not weaken, skip, or delete a failing test to make a change pass.
- Do not silently change the specification to match an implementation defect.
- Do not invent language semantics. Stop and report an ambiguity.
- Preserve experimental work. Classify it accurately before proposing removal
  or redesign.
- No unsupported source type may silently become `Int`, `Float`, `Bool`, or any
  other convenient type.
- Invalid programs must stop before IR or backend generation.
- Treat CPU, ROCm, and CUDA support as separate capabilities. A flag, annotation,
  LLVM text transformation, or helper simulation is not hardware execution.
- Keep benchmark and public claims tied to immutable evidence in
  `claim-verification/`.

## Ownership and change boundaries

- The lead owns language semantics, type/ownership/memory rules, compiler-wide
  architecture, IR and backend contracts, compatibility decisions, final
  integration, and all claims of stability, correctness, safety, or performance.
- A writing agent may change only files explicitly listed in its task contract.
- Coupled parser-to-backend features have one vertical-slice owner.
- Concurrent writing agents must have non-overlapping files and semantics.
- Existing user or agent changes outside the assigned files must be preserved.

## Required workflow

1. Record the task ID, observed behavior, hypothesis, frozen semantics, allowed
   files, acceptance tests, risks, and stop conditions in `TASK_LEDGER.md`.
2. For behavior changes, create or approve a failing regression test first.
3. Make the smallest complete change that satisfies the frozen semantics.
4. Run focused tests and then the baseline gate from repository root:

   ```bash
   ./tools/test.sh
   ```

   On Windows, ensure `%USERPROFILE%\.cargo\bin` is inherited by Git Bash.
5. Update the capability/state/decision documents affected by the result.
6. After an accepted checkpoint is published, synchronize the cumulative draft PR
   title/body to the exact accepted public head, current executable capabilities,
   immutable evidence, and remaining exclusions. Never present unpublished local
   work as accepted, and verify the rendered PR metadata before closing the task.
7. Do not stack new implementation on a red build.

## Evidence and reporting

Every delegated task must report:

1. Finding or implementation summary.
2. Evidence with file and symbol or line references.
3. Files changed.
4. Commands executed.
5. Test results.
6. Remaining uncertainty.
7. Regression risks.
8. Commit SHA, when applicable.
9. Recommended next action.

Read-only auditors must not edit or commit. Implementation agents must stop if
the change unexpectedly crosses more than two compiler phases or requires a
semantic decision not frozen in the task contract.
