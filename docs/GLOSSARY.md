# Glossary

A comprehensive reference of identifiers, terms, and acronyms used throughout the Regen Agentic Tokenomics framework.

## Table of Contents

1. [Mechanism IDs](#mechanism-ids)
2. [Agent IDs](#agent-ids)
3. [Workflow IDs](#workflow-ids)
4. [Governance Process IDs](#governance-process-ids)
5. [Governance Layers](#governance-layers)
6. [Technical Terms](#technical-terms)
7. [Regen Network Terms](#regen-network-terms)
8. [Economic Terms](#economic-terms)
9. [Security Terms](#security-terms)

---

## Mechanism IDs

### Token Utility Mechanisms

| ID | Name | Description |
|----|------|-------------|
| **M001-ENH** | Credit Class Approval Voting Enhancement | Enhances the credit class creator allowlist with tiered approval thresholds, agent pre-screening, and expedited paths for low-risk proposals |
| **M008** | Data Attestation Bonding | Requires attesters to bond REGEN tokens against ecological data claims; bonds can be slashed for false or misleading attestations |
| **M009** | Service Provision Escrow | Enables trustless engagement of ecosystem services (verification, monitoring, methodology development) with milestone-based payment release and dispute resolution |
| **M010** | Reputation / Legitimacy Signaling | Stake-weighted endorsement system where stakeholders signal trust in entities (credit classes, projects, verifiers, addresses) with time-decaying scores and a challenge workflow |
| **M011** | Marketplace Curation & Quality Signals | Curated collections of vetted credit batches with quality scoring; curators stake REGEN and earn a share of trade fees from their collections |

### Economic Reboot Mechanisms

| ID | Name | Description |
|----|------|-------------|
| **M012** | Fixed Cap Dynamic Supply | Replaces inflationary PoS supply with a hard-capped (~221M REGEN) algorithmic mint/burn model tied to ecological activity; inspired by carrying capacity and EIP-1559 |
| **M013** | Value-Based Fee Routing | Replaces flat gas fees with value-proportional fees on ecological credit transactions, routed to four pools: burn, validator fund, community pool, and agent infrastructure |
| **M014** | Authority Validator Governance | Transitions from Proof of Stake (capital-weighted) to Proof of Authority (contribution-weighted) with a curated validator set of 15-21 mission-aligned validators |
| **M015** | Contribution-Weighted Rewards | Replaces passive staking rewards with activity-based distribution from the Community Pool, proportional to ecological and governance contributions |

---

## Agent IDs

| ID | Name | Role | Governance Layer |
|----|------|------|------------------|
| **AGENT-001** | Registry Reviewer | Pre-screens credit class applications, validates project registrations, verifies credit batch issuances, reviews service escrow milestones | Layer 2-3 |
| **AGENT-002** | Governance Analyst | Analyzes and summarizes governance proposals, predicts voting outcomes, publishes post-vote reports | Layer 1 (informational) |
| **AGENT-003** | Market Monitor | Detects price anomalies, monitors liquidity, analyzes retirement patterns, scores marketplace curation quality | Layer 1-2 |
| **AGENT-004** | Validator Monitor | Tracks validator performance and uptime, analyzes delegation flows, monitors network decentralization | Layer 1-3 |

---

## Workflow IDs

### Registry Reviewer (AGENT-001)

| ID | Name | Trigger | Governance Layer |
|----|------|---------|------------------|
| **WF-RR-01** | Credit Class Application Review | `MsgProposeClassCreator` | Layer 2 |
| **WF-RR-02** | Project Registration Validation | `MsgCreateProject` | Layer 1-2 |
| **WF-RR-03** | Credit Batch Issuance Verification | `MsgCreateBatch` (pending verification) | Layer 2 |
| **WF-RR-04** | Service Escrow Milestone Review | `MilestoneSubmitted` (M009) | Layer 1 (advisory) |

### Governance Analyst (AGENT-002)

| ID | Name | Trigger | Governance Layer |
|----|------|---------|------------------|
| **WF-GA-01** | Proposal Analysis & Summarization | `ProposalSubmitted` | Layer 1 |
| **WF-GA-02** | Voting Outcome Prediction & Alerts | `VoteCast` or periodic (every 6h during voting) | Layer 1 |
| **WF-GA-03** | Post-Vote Analysis & Reporting | `ProposalFinalized` | Layer 1 |

### Market Monitor (AGENT-003)

| ID | Name | Trigger | Governance Layer |
|----|------|---------|------------------|
| **WF-MM-01** | Price Anomaly Detection | `SellOrderCreated` or `SellOrderFilled` | Layer 1-2 |
| **WF-MM-02** | Liquidity Monitoring & Reporting | Periodic (every 1h) or significant trade (>$10k) | Layer 1 |
| **WF-MM-03** | Retirement Pattern Analysis | `MsgRetire` | Layer 1 |
| **WF-MM-04** | Curation Quality Monitoring & Scoring | `SellOrderCreated`, periodic (daily), or `CollectionBatchAdded` (M011) | Layer 1-2 |
| **WF-MM-05** | Fee Revenue Monitoring (M013) | `EventFeeCollected`, periodic (every 6h) | Layer 1 |
| **WF-MM-06** | Supply Equilibrium Monitoring (M012) | `EventSupplyMint`, `EventSupplyBurn`, periodic (daily) | Layer 1 |

### Validator Monitor (AGENT-004)

| ID | Name | Trigger | Governance Layer |
|----|------|---------|------------------|
| **WF-VM-01** | Validator Performance Tracking | Periodic (every block) or `SlashEvent` | Layer 1 |
| **WF-VM-02** | Delegation Flow Analysis | `MsgDelegate`, `MsgUndelegate`, or `MsgRedelegate` | Layer 1 |
| **WF-VM-03** | Network Decentralization Monitoring | Periodic (daily) or `ValidatorSetChange` | Layer 1-3 |
| **WF-VM-04** | Authority Validator Performance (M014) | `EventValidatorApproved`, periodic (every 12h) | Layer 1-2 |
| **WF-VM-05** | Reward Distribution Monitoring (M015) | `EventRewardsDistributed`, periodic (daily) | Layer 1 |

---

## Governance Process IDs

| ID | Name | Category | Layer |
|----|------|----------|-------|
| **GOV-001** | Credit Class Creator Allowlist | Registry Governance | Layer 2-3 |
| **GOV-002** | Currency Allow List Addition | Marketplace Governance | Layer 1-2 |
| **GOV-003** | Software Upgrade Proposal | Technical Governance | Layer 2-4 |
| **GOV-004** | Community Pool Spend Proposal | Treasury Governance | Layer 3 (always human-in-loop) |
| **GOV-005** | Parameter Change Proposal | System Governance | Layer 2-4 (varies by parameter category) |

---

## Governance Layers

| Layer | Name | Delegation | Automation | Description |
|-------|------|------------|------------|-------------|
| **Layer 1** | Fully Automated | Smart contracts execute autonomously | 100% | Read-only queries, statistical computation, alerts, dashboard updates. No state changes. |
| **Layer 2** | Agentic + Oversight | Agents propose, humans can override | 85%+ | Agent-recommended actions with a human override window (24-72h). Auto-executes if no override. |
| **Layer 3** | Human-in-Loop | Humans decide, agents assist | ~50% | Agents provide analysis and recommendations; humans make the final decision. Large grants, contested approvals, novel proposals. |
| **Layer 4** | Constitutional | Supermajority governance only | 0% | Protocol upgrades, governance parameter changes, consensus mechanism changes. Requires 67%+ supermajority. |

---

## Technical Terms

| Term | Definition |
|------|------------|
| **PoA** | Proof of Authority -- a consensus model where a curated set of known validators produce blocks, replacing capital-weighted Proof of Stake. Specified in M014. |
| **PoS** | Proof of Stake -- the current Regen Ledger consensus model where validators are weighted by the amount of staked REGEN delegated to them. Being replaced by PoA via M014. |
| **TWAP** | Time-Weighted Average Price -- a pricing method that weights prices by the duration they were valid, reducing manipulation from short-lived price spikes. Used in credit valuation. |
| **cadCAD** | Complex Adaptive Dynamics Computer Aided Design -- a Python library for modeling complex systems. Used for tokenomics simulations and mechanism parameter calibration. |
| **ElizaOS** | An open-source AI agent runtime framework. Used as the runtime for AGENT-001 through AGENT-004, providing character definitions, plugin architecture, and multi-platform integration. |
| **CosmWasm** | A smart contract platform for Cosmos SDK chains. Contracts are written in Rust and compiled to WebAssembly. Used for M008 (Arbiter DAO), M009 (Service Escrow), M010 (Reputation Registry), and M011 (Marketplace Curation). |
| **MCP** | Model Context Protocol -- a protocol for connecting AI agents to external data sources and tools. Three MCP servers are used: KOI MCP (knowledge graph), Ledger MCP (on-chain state), TX Builder MCP (transaction preparation). |
| **KOI** | Knowledge Organization Infrastructure -- a knowledge graph framework built on Apache Jena that provides semantic search, document retrieval, and evidence linking for agent workflows. |
| **OODA** | Observe-Orient-Decide-Act -- a decision-making loop pattern used to structure all 14 agent workflows. Each workflow defines what data to observe, how to orient (analyze), what decision to make, and what action to take. |
| **IRI** | Internationalized Resource Identifier -- a generalized URI used in Regen Ledger to reference off-chain data (methodology documents, monitoring reports, attestation evidence). Resolved via the `x/data` module. |
| **Cosmos SDK** | A framework for building application-specific blockchains. Regen Ledger is built on Cosmos SDK, using modules like `x/ecocredit`, `x/data`, `x/staking`, and `x/gov`. |
| **DAO DAO** | A CosmWasm-based DAO framework for Cosmos chains. Used for the Arbiter DAO (M008/M009 dispute resolution) and potentially for validator governance subDAOs under M014. |
| **Tendermint / CometBFT** | The Byzantine Fault Tolerant consensus engine underlying Cosmos SDK chains, providing block production and finality. |
| **IBC** | Inter-Blockchain Communication -- a Cosmos protocol for cross-chain token transfers and message passing. Relevant to currency allowlist (GOV-002) and cross-chain fee collection. |
| **pgvector** | A PostgreSQL extension for vector similarity search. Used in the agent state database for embedding storage and semantic retrieval. |
| **BGE** | BAAI General Embedding -- a 1024-dimensional embedding model used for semantic similarity in agent knowledge retrieval and duplicate detection. |
| **Redis Streams** | A Redis data structure for event streaming and message queuing. Used for inter-agent coordination and event-driven workflow triggers. |
| **Apache Jena** | An open-source Java framework for semantic web and linked data. Hosts the KOI knowledge graph via Jena Fuseki (SPARQL endpoint). |
| **Buf** | A tool for working with Protocol Buffers. Used for linting and code generation of Cosmos SDK protobuf definitions. |

---

## Regen Network Terms

| Term | Definition |
|------|------------|
| **Credit class** | A type of ecological credit on Regen Ledger, defined by a methodology, credit type, and set of approved issuers. Examples: carbon credits, biodiversity credits. Managed via `x/ecocredit`. |
| **Credit batch** | A specific issuance of credits within a credit class, associated with a project, monitoring period, and quantity. Each batch has a unique denom. |
| **Retirement** | The act of permanently claiming the ecological benefit represented by a credit. Retired credits cannot be transferred or traded. Triggered by `MsgRetire`. |
| **Attestation** | A verifiable claim about ecological data, bonded under M008. Includes project boundaries, baseline measurements, credit issuance claims, and methodology validations. |
| **REGEN** | The native staking and governance token of Regen Network. Used for governance voting, staking (current PoS), service bonds, attestation bonds, reputation signaling, and fee payment. |
| **uregen** | The smallest denomination of REGEN. 1 REGEN = 1,000,000 uregen. All on-chain amounts are denominated in uregen. |
| **Regen Ledger** | The application-specific Cosmos SDK blockchain operated by Regen Network. Hosts the ecocredit module, data module, marketplace, and governance. |
| **Regen Registry** | The program and standards framework for ecological credit origination on Regen Network, including credit class approval, project registration, and credit issuance. |
| **Credit type** | A category of ecological benefit (e.g., carbon, biodiversity). Each credit class is associated with exactly one credit type. |
| **MRV** | Measurement, Reporting, and Verification -- the process of quantifying ecological outcomes, reporting them, and having them independently verified. Central to credit issuance. |
| **Methodology** | A documented procedure for measuring and verifying a specific type of ecological outcome. Referenced by credit classes via IRI. |
| **Verifier** | An entity authorized to verify ecological claims under a methodology. Verifier reputation is tracked by M010. |
| **Project** | A registered ecological initiative on Regen Ledger, associated with a geographic location (GeoJSON boundary) and one or more credit classes. |
| **Community pool** | An on-chain treasury funded by a share of fee revenue (under M013) or block rewards (current PoS). Disbursed via governance proposals (GOV-004) and automatic activity rewards (M015). |
| **Allowlist** | A governance-controlled list of addresses permitted to perform specific actions (e.g., `AllowedClassCreators`, `AllowedDenoms`). Managed via governance proposals. |
| **KOI knowledge graph** | The semantic knowledge base backing agent workflows, built on Apache Jena and accessible via KOI MCP. Stores evidence, analysis artifacts, audit trails. |

---

## Economic Terms

| Term | Definition |
|------|------------|
| **Regrowth rate** | The base rate at which new REGEN tokens are minted under M012. Minting follows `M[t] = r * (C - S[t])`, where `r` is the regrowth rate, `C` is the hard cap, and `S[t]` is current supply. Default: 2% per period. |
| **Hard cap** | The absolute maximum supply of REGEN tokens (~221M), established by M012. Supply can fluctuate below the cap through mint/burn dynamics but can never exceed it. Changeable only via Layer 4 constitutional governance. |
| **Stability tier** | A commitment level in M015 where token holders lock REGEN for defined periods (30/90/180/365 days) in exchange for enhanced reward multipliers and governance weight. Replaces passive staking under PoA. |
| **Activity score** | A composite score computed by M015 that quantifies a participant's contribution to the network. Based on weighted inputs: credit purchases, retirements, service facilitation, and governance participation. |
| **Burn share** | The percentage of fee revenue permanently destroyed (burned) to reduce circulating supply. Part of M013 fee routing. Ranges from 25-35% depending on adopted model. |
| **Validator fund** | A fee-revenue pool (M013) that provides fixed compensation to authority validators (M014), replacing inflationary staking rewards. |
| **Agent infrastructure fund** | A fee-revenue pool (M013) that funds AI agent operational costs (compute, MCP server hosting, knowledge graph maintenance). |
| **Value-based fee** | A transaction fee proportional to the ecological credit value being transacted, replacing flat gas fees. Specified in M013. Rates vary by transaction type (1-3% issuance, 0.1% transfer, 0.5% retirement, 1% trade). |
| **Carrying capacity** | An ecological metaphor for M012's hard cap: the upper limit of token supply, analogous to nature's limit on population within an ecosystem. Supply contracts and expands within this bound. |
| **Ecological multiplier** | A factor in M012's regrowth formula that adjusts minting rate based on real-world ecological metrics (e.g., CO2 levels). Disabled in v0; requires oracle integration. Range: [0.0, 1.0+]. |
| **Mint/burn equilibrium** | The state in M012 where minting (regrowth) and burning (from fees) are approximately equal over consecutive periods, indicating a stable token supply. |
| **Platform fee** | A small percentage (default 1%) of escrow value in M009 that goes to the community pool upon milestone approval or agreement completion. |
| **Service bond** | A deposit required from service providers in M009 (10% of escrow value) that can be slashed if the provider fails to deliver. Refunded on successful completion. |
| **Dual-track tally** | The voting model under M014 where proposals are tallied on two tracks: authority validators (60% weight, equal per validator) and token holders (40% weight, contribution-weighted via M015). |

---

## Security Terms

| Term | Definition |
|------|------------|
| **Invariant** | A property that must hold true at all times during contract execution. Each mechanism specifies numbered security invariants (e.g., "Deposit Conservation", "Cap Inviolability") that implementations must preserve. |
| **Slashing** | The penalty of forfeiting part or all of a bonded deposit. Applied to: attestation bonds (M008) for false claims, service provider bonds (M009) for delivery failure, and reputation signals (M010 v1) for invalid endorsements. |
| **Bond** | Tokens locked as collateral to back a claim or commitment. Types include: attestation bonds (M008, 500-5000 REGEN by type), service bonds (M009, 10% of escrow), and agent service bonds (10,000 REGEN). |
| **Escrow** | Tokens locked in a smart contract and released upon satisfaction of predefined conditions. Used in M001-ENH (proposal deposits), M009 (service payments), and M011 (curator stakes). |
| **Arbiter DAO** | A DAO DAO subDAO responsible for resolving disputes in M008 (attestation challenges) and M009 (service escrow disputes). Members are staked in an arbiter pool and must be neutral (no relationship to disputing parties). |
| **Override window** | A time period (24-72h) during which humans can override an agent's automated decision at Layer 2. If no override occurs, the agent's recommendation auto-executes. |
| **Challenge** | A formal dispute against an attestation (M008) or reputation signal (M010). Challengers must provide evidence and a deposit (10% of the bond or challenger's stake). Failed challenges forfeit the deposit. |
| **Challenge window** | The time period during which an attestation or signal can be challenged. Varies by type: 60-600 days for attestations (M008), configurable for reputation signals (M010). |
| **Human override** | The ability for human governance to supersede any agent recommendation. Guaranteed by the governance layer model: agents can never directly approve or reject without a human override window. |
| **Deposit conservation** | A security invariant (M001-ENH) ensuring that the sum of all deposits equals the sum of all refunds, slashes, and remaining escrow balance. No tokens are created or destroyed in the deposit flow. |
| **Confidence threshold** | The minimum confidence score (0.0-1.0) an agent must reach before its recommendation is acted upon. Layer 2 operations require 0.85; below-threshold recommendations escalate to human review. |
| **Veto** | A governance vote option that blocks a proposal regardless of approval votes. Either track (validators or holders) can veto at >33.4% under the dual-track tally model (M014). |
| **Quorum** | The minimum participation required for a governance vote to be valid. Under M014 dual-track: 2/3 of authority validators and 10% of circulating supply for the holder track. |
| **Supermajority** | A voting threshold of 67%+ required for Layer 4 (constitutional) governance changes, including hard cap modifications, consensus mechanism changes, and governance parameter updates. |

---

*This document is part of the Regen Network Agentic Tokenomics framework.*
