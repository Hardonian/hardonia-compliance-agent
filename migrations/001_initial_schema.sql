-- Compliance Agent Database Schema
-- Initial migration: creates all core tables

-- Tenants (fintech/healthtech/insurtech companies using the agent)
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    industry VARCHAR(100) NOT NULL, -- 'fintech', 'healthtech', 'insurtech'
    jurisdictions TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Regulatory Sources (where we fetch changes from)
CREATE TABLE IF NOT EXISTS regulatory_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    source_type VARCHAR(100) NOT NULL, -- 'recall_intelligence', 'trade_compliance', 'sec', 'finra'
    jurisdiction VARCHAR(50) NOT NULL DEFAULT 'us',
    categories TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_checked TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Regulatory Changes (fetched from sources)
CREATE TABLE IF NOT EXISTS regulatory_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES regulatory_sources(id),
    title VARCHAR(500) NOT NULL,
    summary TEXT NOT NULL,
    jurisdiction VARCHAR(50) NOT NULL,
    category VARCHAR(100) NOT NULL, -- 'aml', 'sanctions', 'data_privacy', 'consumer_protection'
    change_type VARCHAR(50) NOT NULL, -- 'new_regulation', 'amendment', 'enforcement_action', 'guidance'
    severity VARCHAR(20) NOT NULL, -- 'critical', 'high', 'medium', 'low'
    effective_date DATE,
    affected_entities TEXT[] NOT NULL DEFAULT '{}',
    raw_data JSONB,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Agent Runs (audit trail of agent executions)
CREATE TABLE IF NOT EXISTS agent_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'running', -- 'running', 'completed', 'partial', 'failed'
    sources_checked INTEGER NOT NULL DEFAULT 0,
    changes_found INTEGER NOT NULL DEFAULT 0,
    tasks_created INTEGER NOT NULL DEFAULT 0,
    errors JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Compliance Tasks (generated from regulatory changes)
CREATE TABLE IF NOT EXISTS compliance_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    regulatory_change_id UUID NOT NULL REFERENCES regulatory_changes(id),
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    priority VARCHAR(20) NOT NULL, -- 'critical', 'high', 'medium', 'low'
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'in_progress', 'completed', 'overdue', 'cancelled'
    assigned_to VARCHAR(255),
    due_date DATE,
    completed_at TIMESTAMPTZ,
    evidence_required TEXT[] NOT NULL DEFAULT '{}',
    evidence_collected JSONB NOT NULL DEFAULT '{}',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Compliance Evidence (uploaded evidence files)
CREATE TABLE IF NOT EXISTS compliance_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES compliance_tasks(id),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    filename VARCHAR(500) NOT NULL,
    file_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    storage_path VARCHAR(1000) NOT NULL,
    sha256_hash VARCHAR(64) NOT NULL,
    uploaded_by VARCHAR(255),
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_regulatory_changes_source_id ON regulatory_changes(source_id);
CREATE INDEX IF NOT EXISTS idx_regulatory_changes_fetched_at ON regulatory_changes(fetched_at DESC);
CREATE INDEX IF NOT EXISTS idx_compliance_tasks_tenant_id ON compliance_tasks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_compliance_tasks_status ON compliance_tasks(status);
CREATE INDEX IF NOT EXISTS idx_compliance_tasks_due_date ON compliance_tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_compliance_evidence_task_id ON compliance_evidence(task_id);
CREATE INDEX IF NOT EXISTS idx_agent_runs_started_at ON agent_runs(started_at DESC);

-- Insert default regulatory sources
INSERT INTO regulatory_sources (name, source_type, jurisdiction, categories) VALUES
    ('SEC EDGAR', 'recall_intelligence', 'us', '{"securities", "corporate_filing"}'),
    ('FINRA', 'recall_intelligence', 'us', '{"broker_dealer", "investment"}'),
    ('CFPB', 'recall_intelligence', 'us', '{"consumer_protection", "financial"}'),
    ('OFAC Sanctions', 'trade_compliance', 'us', '{"sanctions", "aml"}'),
    ('BSA/AML Updates', 'trade_compliance', 'us', '{"aml", "bank_secrecy"}'),
    ('GDPR Updates', 'recall_intelligence', 'eu', '{"data_privacy", "gdpr"}'),
    ('FCA Guidance', 'recall_intelligence', 'gb', '{"financial_regulation", "conduct"}')
ON CONFLICT DO NOTHING;