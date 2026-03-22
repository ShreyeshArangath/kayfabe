# Unified Code Review Output Format

This is the format to produce after synthesizing all 4 subagent results. Group findings by severity, not by agent.

## Template

```markdown
# Code Review

## Critical (Ship-Blockers)

### Issue Title — file.ext:LINE [source-agent]

**Why it matters**: One sharp sentence on the blast radius.

**Before:**
```lang
current code
```

**After:**
```lang
fixed code
```

---

## High Priority

### Issue Title — file.ext:LINE [source-agent]

**Why it matters**: ...

**Before:**
```lang
current code
```

**After:**
```lang
fixed code
```

---

## Medium

### Issue Title — file.ext:LINE [source-agent]

**Description**: What's wrong and how to fix it.

---

## Low / Polish

- **Issue** — file.ext:LINE [source-agent]: Brief description.

---

## Scorecard

| Severity | Count |
|----------|-------|
| Critical | N     |
| High     | N     |
| Medium   | N     |
| Low      | N     |

## What's Working

Call out genuinely good choices — be specific, not generic. Examples:
- "Good use of context cancellation in the gRPC handler"
- "Error types are well-structured and provide actionable messages"
NOT: "The code looks clean" or "Good job overall"

## Quick Wins

Highest-impact, lowest-effort changes in priority order:
1. ...
2. ...
3. ...
```

## Rules

- **Before/After code** is REQUIRED for Critical and High findings
- **Before/After code** is OPTIONAL for Medium and Low
- **Source agent tags**: `[bug-catcher]`, `[style-enforcer]`, `[systems-auditor]`, `[ux-advocate]`
- If a finding was flagged by multiple agents, list all: `[bug-catcher, systems-auditor]`
- **What's Working** must cite specific code decisions, not generic praise
- **Quick Wins** should be ordered by impact/effort ratio
- Omit empty severity sections entirely (don't show "## Critical" with no findings)
- Omit "What's Working" only if genuinely nothing stands out
