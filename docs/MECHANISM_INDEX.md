# Mechanism index generation

This repo includes a deterministic script to keep the `README.md` mechanism list up to date.

## Update
```bash
node scripts/build-mechanism-index.mjs
```

## Check (CI-friendly)
```bash
node scripts/build-mechanism-index.mjs --check
```

The script generates the section between:
- `<!-- BEGIN MECHANISMS INDEX -->`
- `<!-- END MECHANISMS INDEX -->`
