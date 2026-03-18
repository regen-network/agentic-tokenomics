# Community Background & Forum Discussion Index

## Purpose

This document provides the community context and alignment evidence behind the PoA migration and economic reboot mechanisms (M012–M015). The mechanism specs are technically precise but lack the 2+ years of community deliberation history that grounds them. Reviewers and new contributors need this context.

---

## Timeline of Community PoA Discussions

| Date | Event | Forum Thread |
|------|-------|--------------|
| Apr 2021 | Mainnet launch with 50 validators (PoS) | — |
| Jul 2021 | Proposal #3: Increase validator seats to 75 | [#256](https://forum.regen.network/t/proposal-3-increase-validators-seats/256) |
| Jan 2024 | "Lowering Inflation" proposal passed | Tokenomics WG |
| Mar 2024 | Proposal #41: Halving REGEN inflation adopted | Tokenomics WG |
| Jun 2024 | **PoA Consensus RFC posted** — first formal proposal | [#70](https://forum.regen.network/t/regen-network-proof-of-authority-consensus-rfc/70) |
| Jul 2024 | Cosmos Shared Security RFC (ICS alternative explored) | [#67](https://forum.regen.network/t/cosmos-shared-security-rfc/67) |
| Jul 2024 | Cross-chain ecoLedger integration — validator competence concerns raised | [#55](https://forum.regen.network/t/cross-chain-integration-with-ecoledger/55) |
| Mid-2024 | PoA socialized with existing validators | (Telegram/calls) |
| 2024 | RFC: Reduce validator set from 75 to 21 | [#59](https://forum.regen.network/t/rfc-proposal-draft-to-reduce-active-validator-set-to-21-validators/59) |
| Feb 2025 | Fixed Cap, Dynamic Supply proposal posted | [#34](https://forum.regen.network/t/fixed-cap-dynamic-supply/34) |
| Oct 2025 | Governance findings audit published | [#539](https://forum.regen.network/t/regen-s-governance-findings-for-decentralized-and-regenerative-daos/539) |
| Oct 2025 | Tokenomics architecture checklist summary | [#538](https://forum.regen.network/t/regens-tokenomics-architecture-checklist-summary/538) |
| Dec 2025 | **Economic Reboot Roadmap v0.1** — 4 workstreams synthesized | [#567](https://forum.regen.network/t/regen-economic-reboot-roadmap-v0-1/567) |
| Jan 2026 | **Comprehensive Governance & Economic Architecture Upgrade** drafted | [Notion](https://www.notion.so/Draft-Comprehensive-Proposal-Regen-Network-Governance-Economic-Architecture-Upgrade-2fb25b77eda180af8742debdfaed0b3c) |
| Jan 2026 | Tokenomics WG discusses 3 PoA options; Gregory confirms 18 months of prior socialization | [#19/64](https://forum.regen.network/t/regen-tokenomics-wg/19/64) |
| Jan 2026 | Gregory references Cosmos Labs native PoA tooling | [#567/4](https://forum.regen.network/t/regen-economic-reboot-roadmap-v0-1/567/4) |
| Feb 2026 | **Regen Ledger v7.0** ships (SDK v0.53.4) — one version from native PoA | [#574](https://forum.regen.network/t/software-upgrade-proposal-regen-ledger-v7-0/574) |
| Feb 2026 | agentic-tokenomics repo created with M012–M015 specs | This repo |

---

## Key Community Positions

### Will Szal (Regen Foundation)
- Original PoA proposal author (June 2024)
- Proposed: 7 validators, halt emissions, maintain permissionless governance
- Emphasized pre-vote socialization with validators

### Gregory Landua (RND)
- Suggested 11–17 validators (decentralization value)
- Characterized PoA as return to founding vision: Regen was originally a "consortium blockchain model where a group of trusted entities would run the chain"
- Confirmed 18+ months of community discussion by January 2026
- Referenced Cosmos Labs native PoA tooling
- Leans toward contributor-first fee distribution (Model B) over burn-heavy model

### Max Semenchuk (Tokenomics WG Steward)
- Led WG since August 2023
- Authored original PoA RFC analysis (pros/cons)
- Drove Fixed Cap Dynamic Supply initiative with BlockScience
- Coordinated Shared Security RFC exploration

### brawlaphant (Community)
- Authored Economic Reboot Roadmap synthesizing all workstreams
- Created the agentic-tokenomics repo structure and meta-pack
- Initially opposed Fixed Cap timing ("real market volume should precede redesign")
- Later reengaged positively, proposed coordinating with PoA migration

### James Bettauer (ecoToken)
- Warned that zeroing emissions could cause validator exodus
- Important perspective on cross-chain credit distribution validator competence

### Community Governance Audit ("inca", October 2025)
- 65% voting power concentration in top delegates
- 25% average voter turnout
- 30–35% proposal success rate
- Nakamoto Coefficient 18–22 (moderate decentralization)
- Geographic clustering in North America/Europe
- All findings directly support the case for PoA

---

## Historical Validator Set Trajectory

```
2021: 50 validators at launch
       ↓ (Proposal #3)
2021: 75 validators (optimistic expansion)
       ↓ (declining token price, validator economics)
2024: RFC to reduce to 21 validators
       ↓ (continued decline)
2025-26: ~21 active (frequently dropping below), all operating at a loss
       ↓ (PoA consensus emerging)
2026+: 15–21 curated PoA authorities (proposed)
```

This trajectory illustrates the network's journey from PoS idealism to pragmatic mission-aligned governance.

---

## Current Validator Reality (as of Jan 2026)

- Maximum validator set: 75
- Active validators: ~21 (sometimes dropping below)
- **All validators operate at a loss** — participation is mission-aligned, not economically rational
- Validator rewards insufficient to cover infrastructure costs
- Token price too low to make PoS security economically meaningful

---

## The Founding Vision

From the PoA Consensus RFC (Gregory_RND):

> "[Regen was originally conceived as] a consortium blockchain model where a group of trusted entities would run the chain until a valid Proof of Regeneration model could be created. Later on, the Regen Network project compromised with the prevailing [Cosmos PoS model]."

The PoA migration is framed not as centralization, but as a **return to the founding vision** — now with 5 years of operational experience and a mature community.

---

## Related

- M014 spec: `phase-2/2.6-economic-reboot-mechanisms.md`
- Cosmos Labs x/poa: `docs/cosmos-poa-module.md`
- Bioregional validator framework: `docs/bioregional-validators.md`
