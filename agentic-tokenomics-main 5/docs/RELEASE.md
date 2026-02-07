# Release process

This repo uses a lightweight release discipline.

## When to cut a tag
- After a set of mechanisms/spec updates have merged and WG has reviewed.
- After validator/schema changes that downstream repos depend on.

## Steps
1) Update `CHANGELOG.md` (move items from "Unreleased" to a dated section).
2) Create an annotated tag:
```bash
git tag -a vX.Y.Z -m "agentic-tokenomics vX.Y.Z"
git push origin vX.Y.Z
```

## Notes
- Keep releases conservative: downstream automation depends on stable schemas and paths.
