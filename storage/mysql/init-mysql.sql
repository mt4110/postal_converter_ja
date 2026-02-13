-- Create zip_code_db database (if not exists)
CREATE DATABASE IF NOT EXISTS zip_code_db CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;
USE zip_code_db;

-- Create postal_codes table (if not exists)
CREATE TABLE IF NOT EXISTS postal_codes (
    zip_code CHAR(7) NOT NULL COMMENT '郵便番号',
    prefecture_id SMALLINT NOT NULL COMMENT '都道府県ID',
    city_id VARCHAR(10) NOT NULL COMMENT '市区町村コード',
    prefecture VARCHAR(32) NOT NULL COMMENT '都道府県',
    city VARCHAR(50) NOT NULL COMMENT '市区町村',
    town VARCHAR(500) COMMENT '町名',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '作成日時',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新日時',
    PRIMARY KEY (zip_code, prefecture_id, city, town)
);

-- Create index for zip_code and town
CREATE INDEX idx_postal_codes_zip_code ON postal_codes (zip_code, town);

-- Create audit table for crawler updates
CREATE TABLE IF NOT EXISTS data_update_audits (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    data_version VARCHAR(32) NOT NULL UNIQUE,
    database_type VARCHAR(16) NOT NULL,
    source_url TEXT NOT NULL,
    run_started_at DATETIME NOT NULL,
    run_finished_at DATETIME NOT NULL,
    batch_timestamp DATETIME NOT NULL,
    records_in_feed BIGINT NOT NULL,
    inserted_count BIGINT NOT NULL,
    updated_count BIGINT NOT NULL,
    deleted_count BIGINT NOT NULL,
    total_count BIGINT NOT NULL,
    status VARCHAR(16) NOT NULL,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_data_update_audits_created_at (created_at)
);

-- Create snapshot table for version rollback
CREATE TABLE IF NOT EXISTS postal_codes_snapshots (
    data_version VARCHAR(32) NOT NULL,
    zip_code CHAR(7) NOT NULL,
    prefecture_id SMALLINT NOT NULL,
    city_id VARCHAR(10) NOT NULL,
    prefecture VARCHAR(32) NOT NULL,
    city VARCHAR(50) NOT NULL,
    town VARCHAR(500),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    snapshot_created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (data_version, zip_code, prefecture_id, city, town),
    INDEX idx_postal_codes_snapshots_version (data_version)
);
