-- produce_ai_keys: Produce AI Key 表（用户访问系统的 API Key）
CREATE TABLE produce_ai_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    produce_ai_key_hash VARCHAR(255) NOT NULL UNIQUE,
    produce_ai_key_preview VARCHAR(20) NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_produce_ai_keys_tenant ON produce_ai_keys(tenant_id);
CREATE INDEX idx_produce_ai_keys_user ON produce_ai_keys(user_id);
CREATE INDEX idx_produce_ai_keys_hash ON produce_ai_keys(produce_ai_key_hash);
CREATE INDEX idx_produce_ai_keys_revoked ON produce_ai_keys(revoked) WHERE revoked = FALSE;
