# PR #11 Merge Plan — "Docs: add contributor navigation and architecture map"

## PR Summary

**PR:** https://github.com/regen-network/agentic-tokenomics/pull/11
**Author:** brawlaphant
**State:** Open, mergeable (clean)
**Stats:** 137 files changed, +47,415 lines, 5 commits

## The Problem

The PR's **intended scope** is 4 documentation files (per the author's own comment):

| File | Purpose |
|------|---------|
| `docs/CONTRIBUTOR_NAV.md` | Contributor navigation guide: directory conventions, "one subtree per PR" rule, PR description template |
| `docs/DEPENDENCIES.md` | Dependency cross-reference policy: links-over-vendoring, koi-research relationship |
| `docs/architecture/STACK_MAP.md` | Architecture map with embedded Mermaid diagram showing repo relationships |
| `docs/architecture/STACK_MAP.mmd` | Standalone Mermaid source for the stack map |

Additionally there is a `README_SNIPPET.md` at root — a 3-line snippet suggesting a "Contributing" section pointing to `CONTRIBUTOR_NAV.md`.

**However**, the PR also carries ~130+ artifact files across duplicate directories:

- `agentic-tokenomics-main 2/` — duplicate phase-1, phase-2, mechanisms content
- `agentic-tokenomics-main 3/` (implied by numbering)
- `agentic-tokenomics-main 4/` — adds reference-impl content
- `agentic-tokenomics-main 5/` — adds phase-3, scripts, schemas, changelog
- `agentic-tokenomics-m010-phase1.zip` — binary archive

These "agentic-tokenomics-main N" directories are **upload artifacts** from the contributor's prior work iterations. Their content duplicates what already exists on `main` (merged via PRs #1, #2, #4, #9, #10). **They must not be merged.**

## Content Quality Assessment

The 4 intended docs files are **good and useful**:

1. **CONTRIBUTOR_NAV.md** — Establishes clear "one subtree per PR" discipline, defines directory conventions, and includes a PR description template. Directly addresses the exact problem this PR itself suffers from (sprawling multi-subtree changes). Well-structured.

2. **DEPENDENCIES.md** — Codifies the links-over-vendoring policy and documents koi-research as a primary dependency. Practical guidance for keeping PRs small.

3. **STACK_MAP.md** — Mermaid architecture diagram mapping agentic-tokenomics ↔ koi-research ↔ regen-ai-claude ↔ regen-heartbeat relationships. Useful navigation aid for contributors and agents. Includes repo links.

4. **STACK_MAP.mmd** — Standalone Mermaid source. Clean, matches the embedded diagram.

**Minor issues to consider:**
- `README_SNIPPET.md` should not be merged as-is (a standalone snippet file at root is odd). The Contributing pointer could instead be added directly to the existing `README.md` or `CONTRIBUTING.md`.
- The Gemini bot flagged 6 review comments on non-docs files — these are all on artifact files that won't be merged, so they're moot.

## Merge Options

### Option A: Cherry-pick the docs-only commits (Recommended)

Commits 4 (`401e518`) and 5 (`9990d88`) have docs-only commit messages. If they cleanly contain only the 4 docs files:

```bash
git fetch <fork-remote> main
git cherry-pick 401e5182f6e62f0806e140f438ef6e6926c66cd9
git cherry-pick 9990d88d6c7a1fefdf84af8883d1a7f5b30839e5
```

**Risk:** These commits may also include artifact files if the contributor staged everything. Needs verification.

### Option B: Manual extraction to a clean branch (Safest)

1. Create a clean branch from `main`
2. Extract only the 4 docs files from the PR branch
3. Optionally add a Contributing line to `README.md` (from `README_SNIPPET.md` intent)
4. Commit with attribution (`Co-authored-by: brawlaphant`)
5. Open a new clean PR or merge directly
6. Close PR #11 with a comment explaining what was done

```bash
git checkout -b docs/contributor-nav-and-stack-map main
git checkout origin/pr/11 -- docs/CONTRIBUTOR_NAV.md docs/DEPENDENCIES.md docs/architecture/STACK_MAP.md docs/architecture/STACK_MAP.mmd
git add docs/
git commit -m "Docs: add contributor navigation and architecture map

Co-authored-by: brawlaphant <brawlaphant@users.noreply.github.com>

Extracted from PR #11. Only the intended docs/ changes are included;
artifact directories (agentic-tokenomics-main 2-5/, zip) excluded."
```

### Option C: Squash-merge + immediate cleanup (Not recommended)

Squash-merge the full PR, then immediately delete artifact directories. This puts 47K lines of junk into git history even after cleanup. Avoid.

### Option D: Ask contributor to resubmit clean (Slowest)

Comment on the PR asking brawlaphant to open a new PR from a clean branch with only the docs/ changes. This is the "correct" workflow but may lose momentum.

## Recommended Plan: Option B (Manual extraction)

**Rationale:** Option B is the safest. It guarantees no artifact files leak in, preserves attribution, and doesn't depend on the commit boundaries being clean. It also doesn't require waiting on the contributor.

### Execution steps:

1. **Comment on PR #11** acknowledging the docs contribution, explaining the merge approach
2. **Create clean branch** from current `main`
3. **Extract the 4 docs files** from the PR (copy content, not cherry-pick)
4. **Optionally** add a Contributing pointer to `README.md` (inspired by `README_SNIPPET.md`)
5. **Commit** with `Co-authored-by: brawlaphant` attribution
6. **Merge** to `main` (direct push or clean PR)
7. **Close PR #11** with a comment linking to the merge commit
8. **Do NOT merge** `README_SNIPPET.md`, the `.zip`, or any `agentic-tokenomics-main N/` directories

## Proposed PR #11 Comment

> Thanks for the contributor navigation and stack map additions, @brawlaphant — these are genuinely useful for keeping the repo navigable.
>
> As you noted, this PR carries ~130 artifact files in `agentic-tokenomics-main 2-5/` directories plus a `.zip` archive that shouldn't be merged. Rather than asking you to rebase and clean up, we'll extract the 4 intended `docs/` files into a clean commit with attribution and merge that directly.
>
> **Files being merged:**
> - `docs/CONTRIBUTOR_NAV.md`
> - `docs/DEPENDENCIES.md`
> - `docs/architecture/STACK_MAP.md`
> - `docs/architecture/STACK_MAP.mmd`
>
> **Not being merged:**
> - `README_SNIPPET.md` (the Contributing pointer will be integrated into the existing README instead)
> - `agentic-tokenomics-m010-phase1.zip`
> - All `agentic-tokenomics-main */` directories (duplicate content already on main)
>
> The Gemini review comments are all on artifact files that won't be included, so they're resolved by exclusion.
>
> Closing this in favor of a clean merge commit. Thank you for the contribution!
