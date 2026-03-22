# Synthesis Guide: Deduplication & Prioritization

After collecting results from all 4 subagents, follow these rules to produce the unified review.

## Deduplication Rules

1. **Same file + same line + related concern**: Merge into one finding.
   - Example: bug-catcher flags a race condition, systems-auditor flags the same concurrency issue -> merge, note both perspectives, use higher severity.
   - Tag the merged finding with both agents: `[bug-catcher, systems-auditor]`

2. **Same file + different lines + same root cause**: Merge into one finding covering the broader issue.
   - Example: style-enforcer flags inconsistent naming on lines 12, 45, and 78 -> one finding about the naming pattern.

3. **Different files + same pattern**: Keep as separate findings but note the pattern.
   - Example: bug-catcher finds missing null checks in 3 different files -> 3 findings, but note "This is a recurring pattern across the changeset."

4. **Genuinely distinct issues on the same line**: Keep separate.
   - Example: bug-catcher flags a logic error on line 50, style-enforcer flags naming on the same line -> two separate findings.

## Severity Definitions

| Severity | Definition | Examples |
|----------|------------|---------|
| **Critical** | Data loss, security breach, crash in production, cascading failure | SQL injection, unhandled null deref on hot path, resource exhaustion, auth bypass |
| **High** | Bug that will manifest under normal use, significant operational risk | Off-by-one in business logic, missing retry on critical call, misleading error that blocks users |
| **Medium** | Edge case bug, style deviation from established convention, suboptimal resilience | Unicode edge case, inconsistent naming, missing metrics on secondary path |
| **Low** | Polish, minor nit, defense-in-depth suggestion | Slightly verbose code, optional config improvement, documentation suggestion |

## Severity Override Rules

- If bug-catcher rates something Critical and systems-auditor rates the same issue High -> use Critical
- Always use the HIGHER severity when merging findings
- Never downgrade a security finding below High

## Ordering

Within each severity level, order findings by:
1. Security issues first
2. Correctness issues second
3. Distributed systems issues third
4. Style issues fourth
5. UX issues fifth

## Handling Empty Agent Results

- If an agent returns "No [X] found/concerns/issues", omit that agent's category entirely from the output
- Do NOT include a section saying "The systems-auditor found no issues"
- If ALL agents return empty, the review should say: "Clean changeset. No issues found across correctness, style, systems, and UX review."

## Agent Attribution

Every finding MUST include a source agent tag in brackets. This provides transparency about which lens surfaced the issue:
- `[bug-catcher]` — correctness/security finding
- `[style-enforcer]` — convention alignment finding
- `[systems-auditor]` — distributed systems/ops finding
- `[ux-advocate]` — UX/DX finding
- `[bug-catcher, systems-auditor]` — merged cross-agent finding
