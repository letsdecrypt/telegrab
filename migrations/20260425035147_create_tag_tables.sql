-- Add migration script here
-- Tag table
CREATE TABLE tag (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Album-Tag relationship table (many-to-many)
CREATE TABLE album_tag (
    album_id INTEGER NOT NULL REFERENCES doc(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tag(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (album_id, tag_id)
);

-- Index for faster tag lookups
CREATE INDEX idx_tag_name ON tag(name);
CREATE INDEX idx_album_tag_album_id ON album_tag(album_id);
CREATE INDEX idx_album_tag_tag_id ON album_tag(tag_id);
