CREATE TABLE IF NOT EXISTS feed
(
    id           TEXT PRIMARY KEY NOT NULL,
    title        TEXT NOT NULL,
    url          TEXT NOT NULL,
    channel      TEXT NOT NULL,
    published_at TEXT NOT NULL,
    FOREIGN KEY (channel) REFERENCES subscriptions(channel_id)
);
