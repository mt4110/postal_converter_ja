-- zip_code_dbデータベースを作成（存在しない場合のみ）
CREATE DATABASE zip_code_db WITH ENCODING 'UTF8';

-- 使用するデータベースを指定
\c zip_code_db;

-- postal_codesテーブルの作成（存在しない場合のみ）
CREATE TABLE IF NOT EXISTS postal_codes (
    zip_code CHAR(7) NOT NULL, -- 郵便番号
    prefecture_id SMALLINT NOT NULL, -- 都道府県ID
    prefecture VARCHAR(32) NOT NULL, -- 都道府県
    city VARCHAR(50) NOT NULL, -- 市区町村
    town VARCHAR(500), -- 町名
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP, -- 作成日時
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP, -- 更新日時
    PRIMARY KEY (zip_code)
);