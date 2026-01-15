CREATE TABLE IF NOT EXISTS passkeys (
    id VARCHAR(255) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    passkey_json TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_passkeys_user (user_id)
);
