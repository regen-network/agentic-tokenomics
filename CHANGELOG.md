# Changelog

## Unreleased
### Added
- Units 11–20: canonical schemas, mechanism index generator, consumers mapping, WG bulk pack, repo templates, and verification scripts.
- m010 reference implementation vector verifier (`scripts/verify-m010-reference-impl.mjs`) with challenge replay coverage.

### Changed
- m010 scoring now excludes non-contributing signal states when `status` is present (`active`/`resolved_valid` only contribute).
- m010 KPI computation now emits `challenge_kpis` when challenge data is provided.
- m010 replay dataset/schema alignment expanded to cover challenge fixtures and lifecycle statuses (including `escalated`).

### Notes
- This repo is primarily specification content; changes are intended to be deterministic and offline-friendly.
