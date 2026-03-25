# Claims Engine Architecture

## Overview

The Claims Engine validates ecological data against predefined criteria, issues Ecological State Claims, and manages the full lifecycle from raw measurement to tradeable credit.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CLAIMS ENGINE ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  DATA SOURCES              CLAIMS ENGINE              ON-CHAIN STATE        │
│  ┌─────────────┐          ┌─────────────┐           ┌─────────────┐        │
│  │ Field       │          │ Validation  │           │ x/data      │        │
│  │ Measurements│──────────▶│ Pipeline   │──────────▶│ Anchoring   │        │
│  │             │          │             │           │             │        │
│  │ Remote      │          │ Schema      │           │ x/ecocredit │        │
│  │ Sensing     │──────────▶│ Conformance│──────────▶│ Issuance    │        │
│  │             │          │             │           │             │        │
│  │ Lab         │          │ Evidence    │           │ Metadata    │        │
│  │ Results     │──────────▶│ Linking    │──────────▶│ Graph       │        │
│  └─────────────┘          └─────────────┘           └─────────────┘        │
│                                                                             │
│                           ┌─────────────┐                                   │
│                           │ Knowledge   │                                   │
│                           │ Graph (KOI) │◀────── Semantic Indexing          │
│                           │             │                                   │
│                           └─────────────┘                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Concepts

### 1. Ecological State Claim

A verifiable assertion about ecological state at a specific time and place.

```yaml
EcologicalStateClaim:
  claim_id: "esc-2026-001"
  claim_type: "soil_organic_carbon"

  subject:
    project_id: "C06-4997"
    location:
      type: "Polygon"
      coordinates: [[[...coordinates...]]]
    temporal_bounds:
      start: "2025-01-01"
      end: "2025-12-31"

  assertion:
    metric: "SOC_percentage"
    value: 3.42
    unit: "percent"
    depth_range: "0-30cm"
    methodology: "regen:soil-carbon-v1.2.2"

  evidence:
    - type: "lab_analysis"
      iri: "regen:13toVfvfM5B7yuJqq8h3iVRHp3PKUJ4ABxHyvn4MeUMwwv1pWQGL295.rdf"
      confidence: 0.95

    - type: "remote_sensing"
      iri: "regen:sentinel2_ndvi_2025_Q4.json"
      confidence: 0.82

  attestations:
    - attestor: "regen1abc...xyz"
      timestamp: "2026-01-15T10:30:00Z"
      signature: "..."

  status: "verified"
  on_chain_anchor:
    iri: "regen:13toVfvfM5B7yuJqq8h3iVRHp3PKUJ4ABxHyvn4MeUMwwv1pWQGL295.rdf"
    tx_hash: "ABC123..."
    block_height: 12345678
```

### 2. Evidence Hierarchy

```
Evidence Types (by strength):
├── Primary Evidence
│   ├── Lab Analysis Results (highest confidence)
│   ├── Certified Field Measurements
│   └── Peer-Reviewed Data
├── Supporting Evidence
│   ├── Remote Sensing Products
│   ├── Model Outputs
│   └── Historical Baselines
└── Contextual Evidence
    ├── Practice Documentation
    ├── Stakeholder Attestations
    └── Third-Party Verifications
```

### 3. Claim Lifecycle

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Draft   │───▶│ Submitted│───▶│ Validated│───▶│ Attested │───▶│ Anchored │
└──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
     │               │               │               │               │
     │ Creator       │ Automated     │ Human or      │ Cryptographic │ On-chain
     │ prepares      │ schema        │ agent         │ signatures    │ immutable
     │ claim         │ validation    │ verification  │ collected     │ record
```

---

## On-Chain Integration

### x/data Module Usage

The `x/data` module provides content-addressed storage with attestation capabilities:

- **MsgAnchor**: Anchor data hash on-chain
- **MsgAttest**: Attest to data validity
- **ContentHash**: Raw or Graph content hashes

### IRI Construction

Regen IRIs follow the pattern:
```
regen:<base58_encoded_hash>.<format>
```

Example:
```
regen:13toVfvfM5B7yuJqq8h3iVRHp3PKUJ4ABxHyvn4MeUMwwv1pWQGL295.rdf
```

---

## Integration with regen-data-standards

All claims must conform to schemas defined in [regen-data-standards](https://github.com/regen-network/regen-data-standards):

- **LinkML** framework for semantic schema definition
- **JSON-LD** structured data for interoperability
- **SPARQL-queryable** ecological state claims

---

## Forward Compatibility: regen-econ-ontology

The Claims Engine is designed for forward compatibility with [regen-econ-ontology](https://github.com/glandua/regen-econ-ontology):

| Current Schema | Economic Ontology | Purpose |
|----------------|-------------------|---------|
| `EcologicalStateClaim` | `eco:Claim` | Base claim type |
| `Evidence` | `eco:Attestation` | Proof of claim |
| `CreditBatch` | `eco:Asset` | Value unit |
| `SellOrder` | `eco:Offer` | Market primitive |

---

## References

- [regen-data-standards](https://github.com/regen-network/regen-data-standards)
- [x/data module documentation](https://docs.regen.network/modules/data)
- [Registry 2.0 Vision](https://www.youtube.com/watch?v=P2r0jrrybfI)

---

*This document is part of the Regen Network Agentic Tokenomics framework.*
