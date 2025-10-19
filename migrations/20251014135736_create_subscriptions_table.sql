-- Add migration script here
CREATE TABLE subscriptions(
id uuid NOT NULL,
PRIMARY KEY (id),
email TEXT NOT NULL UNIQUE,
name TEXT NOT NULL,
subscribed_at timestamptz NOT NULL
);

CREATE TABLE words(
id uuid NOT NULL,
PRIMARY KEY (id),
word TEXT NOT NULL UNIQUE,
translation TEXT NOT NULL,
word_type TEXT NOT NULL,
access_count INTEGER NOT NULL,
);
