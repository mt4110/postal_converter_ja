-- Database is created by POSTGRES_DB in docker-compose.yml
-- So we just need to create the table.

-- Create a table for postal codes
CREATE TABLE IF NOT EXISTS postal_codes (
    zip_code CHAR(7) NOT NULL, -- 郵便番号
    prefecture_id SMALLINT NOT NULL, -- 都道府県ID
    city_id VARCHAR(10) NOT NULL, -- 市区町村コード (JIS X 0401/0402)
    prefecture VARCHAR(32) NOT NULL, -- 都道府県
    city VARCHAR(50) NOT NULL, -- 市区町村
    town VARCHAR(500), -- 町名
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP, -- 作成日時
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP, -- 更新日時
    PRIMARY KEY (zip_code, prefecture_id, city, town) -- 複合プライマリーキー
);

-- Create an index for zip_code and town
CREATE INDEX idx_postal_codes_zip_code ON postal_codes (zip_code, town);

-- Create audit table for crawler updates
CREATE TABLE IF NOT EXISTS data_update_audits (
    id BIGSERIAL PRIMARY KEY,
    data_version VARCHAR(32) NOT NULL UNIQUE,
    database_type VARCHAR(16) NOT NULL,
    source_url TEXT NOT NULL,
    run_started_at TIMESTAMPTZ NOT NULL,
    run_finished_at TIMESTAMPTZ NOT NULL,
    batch_timestamp TIMESTAMPTZ NOT NULL,
    records_in_feed BIGINT NOT NULL,
    inserted_count BIGINT NOT NULL,
    updated_count BIGINT NOT NULL,
    deleted_count BIGINT NOT NULL,
    total_count BIGINT NOT NULL,
    status VARCHAR(16) NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_data_update_audits_created_at ON data_update_audits (created_at DESC);

-- Create snapshot table for version rollback
CREATE TABLE IF NOT EXISTS postal_codes_snapshots (
    data_version VARCHAR(32) NOT NULL,
    zip_code CHAR(7) NOT NULL,
    prefecture_id SMALLINT NOT NULL,
    city_id VARCHAR(10) NOT NULL,
    prefecture VARCHAR(32) NOT NULL,
    city VARCHAR(50) NOT NULL,
    town VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    snapshot_created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (data_version, zip_code, prefecture_id, city, town)
);

CREATE INDEX idx_postal_codes_snapshots_version ON postal_codes_snapshots (data_version);
