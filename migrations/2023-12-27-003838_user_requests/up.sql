CREATE TABLE user_requests (
    id INTEGER PRIMARY KEY,
    action_id INT NOT NULL,
    endpoint VARCHAR NOT NULL,
    parameters TEXT NOT NULL,
    created_on TIMESTAMP NOT NULL,
    FOREIGN KEY (action_id) REFERENCES user_actions (id)
);