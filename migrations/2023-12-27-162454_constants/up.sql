CREATE TABLE constants (
    id INTEGER NOT NULL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(name)
);
