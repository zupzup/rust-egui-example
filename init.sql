CREATE TABLE IF NOT EXISTS pets (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    age INTEGER NOT NULL,
    kind TEXT NOT NULL
);

INSERT INTO pets (name, age, kind) VALUES ('minka', 9, 'cat');
INSERT INTO pets (name, age, kind) VALUES ('nala', 7, 'dog');

