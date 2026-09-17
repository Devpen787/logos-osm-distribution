# AGENTS.md

## Prime directive

Move the implementation toward externally verifiable LP-0018 criteria without turning assumptions, mocks, or self-authored evidence into stronger claims than they support.

## Source precedence

When sources disagree, use this order:

1. current canonical `logos-co/lambda-prize/prizes/LP-0018.md`;
2. current upstream Logos tool/module documentation and code;
3. this repository's acceptance matrix and architecture docs;
4. implementation comments and agent notes.

Before a material scope change, testnet deployment, adoption claim, or submission, refresh the canonical prize source.

## Allowed autonomous work

Agents may:

- inspect public upstream repositories and documentation;
- implement code on project branches;
- add unit/integration tests;
- run local builds and tests;
- produce reproducible evidence artifacts;
- draft documentation and release notes;
- prepare proposed upstream issues/PRs for human review;
- update requirement traceability when evidence is concrete.

## Human-gated actions

Do not autonomously:

- open a λPrize solution PR;
- make a public eligibility or prize-winning claim;
- contact independent adopters under the maintainer's identity;
- publish promotional/community posts under the maintainer's identity;
- sign, spend, transfer, or claim funds;
- treat a self-controlled project as independent adoption;
- merge a consequential external upstream PR without human approval.

## Roles

### Builder
Implements one bounded proof target. A Builder must state which requirement IDs the change addresses.

### Verifier
Runs the proof from a clean or isolated state and records exact commands, environment, output, and gaps. The Builder should not be the only Verifier for milestone-level claims.

### Skeptic
Looks specifically for false positives: mock-vs-real confusion, stale source assumptions, hidden central services, dev/portable mismatches, checksum mistakes, unsupported platforms, and adoption-independence problems.

### Evidence packager
Maps requirement -> commit/artifact -> test/build/deployment evidence -> truth label.

## Working rules

1. One mission should have one primary proof target.
2. Prefer the smallest end-to-end slice over broad scaffolding.
3. Preserve raw failure evidence before fixing it.
4. Do not call mocked Storage or LEZ behavior testnet verification.
5. Do not call a registry entry verified until the stored bytes and expected Geofabrik snapshot/checksum are tied together by evidence.
6. Do not count adoption unless the consumer is independent of the submitting team and the integration is inspectable.
7. Keep dev and portable Logos module variants explicit; never silently mix them.
8. Avoid mandatory hosted backends or SaaS dependencies.
9. Every external dependency must have a reason, version/pin strategy, and replacement boundary where practical.
10. Leave the repository easier for the next agent to hydrate.

## Required handoff format

Every substantial handoff should include:

- Mission
- Status
- Requirement IDs
- Source refs / upstream commits
- Branch + commit
- What changed
- Commands run
- Evidence produced
- Verification result
- Known gaps
- Recommended next action
- Human decision required, if any

## Definition of done

A code change is not done because it compiles. The mission definition and relevant acceptance-matrix evidence must also be satisfied or explicitly marked as still missing.
