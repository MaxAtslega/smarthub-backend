CREATE TABLE user_users (
                            id INTEGER NOT NULL PRIMARY KEY,
                            username TEXT NOT NULL,
                            theme INT DEFAULT 1 NOT NULL,
                            birthday DATE NOT NULL,
                            language TEXT DEFAULT "en" NOT NULL,
                            created_on TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);