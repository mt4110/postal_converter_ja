-- Create a database and table for postal codes
CREATE DATABASE zip_code_db WITH ENCODING 'UTF8';

-- Specify the database to use
\c zip_code_db;

-- Create a table for postal codes
CREATE TABLE IF NOT EXISTS postal_codes (
    zip_code CHAR(7) NOT NULL, -- 郵便番号
    prefecture_id SMALLINT NOT NULL, -- 都道府県ID
    prefecture VARCHAR(32) NOT NULL, -- 都道府県
    city VARCHAR(50) NOT NULL, -- 市区町村
    town VARCHAR(500), -- 町名
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP, -- 作成日時
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP, -- 更新日時
    PRIMARY KEY (zip_code, prefecture_id, city, town) -- 複合プライマリーキー
);

-- Create an index for zip_code and town
CREATE INDEX idx_postal_codes_zip_code ON postal_codes (zip_code, town);