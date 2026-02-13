-- Database schema for Regen Agentic Tokenomics agents.
--
-- From phase-2/2.5-data-schema-integration.md, adapted for PostgreSQL
-- with pgvector extension for semantic memory.

-- Enable pgvector
CREATE EXTENSION IF NOT EXISTS vector;

-- Agent Identity
CREATE TABLE agents (
    agent_id UUID PRIMARY KEY,
    name VARCHAR(64) NOT NULL UNIQUE,
    type VARCHAR(32) NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    character_hash BYTEA NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    config JSONB NOT NULL DEFAULT '{}'
);

-- Agent Memory (semantic + episodic)
CREATE TABLE agent_memories (
    memory_id UUID PRIMARY KEY,
    agent_id UUID REFERENCES agents(agent_id),
    memory_type VARCHAR(32) NOT NULL,
    content JSONB NOT NULL,
    embedding VECTOR(1024),
    importance_score FLOAT DEFAULT 0.5,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_memories_agent ON agent_memories(agent_id);
CREATE INDEX idx_memories_type ON agent_memories(memory_type);
CREATE INDEX idx_memories_embedding ON agent_memories
    USING ivfflat (embedding vector_cosine_ops);

-- Workflow Execution Log
CREATE TABLE workflow_executions (
    execution_id UUID PRIMARY KEY,
    agent_id UUID REFERENCES agents(agent_id),
    workflow_id VARCHAR(32) NOT NULL,
    trigger_type VARCHAR(16) NOT NULL,
    trigger_data JSONB NOT NULL,
    status VARCHAR(16) NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    result JSONB,
    governance_layer INTEGER,
    escalated_to VARCHAR(64),
    audit_koi_iri TEXT
);

CREATE INDEX idx_executions_agent ON workflow_executions(agent_id);
CREATE INDEX idx_executions_status ON workflow_executions(status);
CREATE INDEX idx_executions_workflow ON workflow_executions(workflow_id);

-- Agent Decisions (Audit Trail)
CREATE TABLE agent_decisions (
    decision_id UUID PRIMARY KEY,
    execution_id UUID REFERENCES workflow_executions(execution_id),
    agent_id UUID REFERENCES agents(agent_id),
    decision_type VARCHAR(32) NOT NULL,
    subject_type VARCHAR(32) NOT NULL,
    subject_id VARCHAR(128) NOT NULL,
    decision VARCHAR(32) NOT NULL,
    confidence FLOAT NOT NULL,
    rationale TEXT NOT NULL,
    evidence JSONB NOT NULL,
    human_override BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_decisions_agent ON agent_decisions(agent_id);
CREATE INDEX idx_decisions_execution ON agent_decisions(execution_id);
CREATE INDEX idx_decisions_subject ON agent_decisions(subject_type, subject_id);

-- Inter-Agent Messages (for coordination protocol)
CREATE TABLE agent_messages (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    from_agent VARCHAR(32) NOT NULL,
    to_agent VARCHAR(32) NOT NULL,
    pattern VARCHAR(16) NOT NULL,
    type VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    correlation_id UUID,
    confidence FLOAT,
    workflow_execution_id UUID REFERENCES workflow_executions(execution_id),
    priority INTEGER DEFAULT 5,
    ttl_ms INTEGER,
    processed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_messages_to ON agent_messages(to_agent);
CREATE INDEX idx_messages_correlation ON agent_messages(correlation_id);
CREATE INDEX idx_messages_unprocessed ON agent_messages(to_agent)
    WHERE processed_at IS NULL;
