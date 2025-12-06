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