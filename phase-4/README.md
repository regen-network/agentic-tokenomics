# Phase 4: Deployment & Migration

## Overview

Phase 4 manages the deployment of Phase 3 implementations to testnet, followed by a structured mainnet migration. This phase includes environment setup, contract deployment procedures, governance proposal templates, and the PoS→PoA transition runbook.

## Sub-Phases

| Sub-Phase | Focus | Status |
|-----------|-------|--------|
| 4.1 | Testnet Deployment | 🔲 Planned |
| 4.2 | Mainnet Migration | 🔲 Planned |
| 4.3 | Community Onboarding | 🔲 Planned |

## Key Outputs

### Testnet Deployment
- Environment configuration and infrastructure setup
- CosmWasm contract deployment scripts
- Agent staging deployment procedures
- Smoke test and validation checklists

### Mainnet Migration
- Governance proposal templates for M012-M015 activation
- PoS→PoA transition runbook (3 phases)
- Inflation→dynamic supply migration procedures
- Rollback procedures for each mechanism

### Community Onboarding
- Validator application process for PoA
- Stability tier enrollment guide
- Agent interaction guides
- FAQ and communication plan

## Outputs

1. [Testnet Deployment](./4.1-testnet-deployment.md)
2. [Mainnet Migration](./4.2-mainnet-migration.md)
3. [Community Onboarding](./4.3-community-onboarding.md)

## Prerequisites

- Phase 3 specifications complete (smart contracts, agents, testing, security)
- Regen Ledger v5.1+ with CosmWasm enabled
- DAO DAO deployment on Regen testnet
- KOI MCP and Ledger MCP operational in staging
- Security audit completed for M008-M011 CosmWasm contracts
- Economic reboot modules (M012-M015) code reviewed

## Timeline Dependencies

```
Phase 3 Complete
  ↓
Phase 4.1: Testnet Deployment (4-6 weeks)
  ↓
Phase 4.2: Mainnet Migration (staged over 6-12 months)
  ↓
Phase 4.3: Community Onboarding (ongoing, parallel with 4.2)
  ↓
Phase 5: Operations & Evolution
```
