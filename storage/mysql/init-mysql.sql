-- Create zip_code_db database (if not exists)
CREATE DATABASE IF NOT EXISTS zip_code_db CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;
USE zip_code_db;

-- Create postal_codes table (if not exists)
CREATE TABLE IF NOT EXISTS postal_codes (
    zip_code CHAR(7) NOT NULL COMMENT '郵便番号',
    prefecture_id SMALLINT UNSIGNED NOT NULL COMMENT '都道府県ID',
    prefecture VARCHAR(32) NOT NULL COMMENT '都道府県',
    city VARCHAR(50) NOT NULL COMMENT '市区町村',
    town VARCHAR(500) COMMENT '町名',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '作成日時',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新日時',
    PRIMARY KEY (zip_code, prefecture_id, city, town)
);

-- Create index for zip_code and town
CREATE INDEX idx_postal_codes_zip_code ON postal_codes (zip_code, town);