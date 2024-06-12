CREATE TABLE IF NOT EXISTS pets (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    age INTEGER NOT NULL,
    kind_id INTEGER NOT NULL,
    FOREIGN KEY (kind_id)
        REFERENCES pet_kinds(id));

CREATE TABLE IF NOT EXISTS pet_kinds (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

INSERT INTO pet_kinds (name) VALUES ('cat');
INSERT INTO pet_kinds (name) VALUES ('dog');

INSERT INTO pets (name, age, kind_id) VALUES ('minka', 9, 1);
INSERT INTO pets (name, age, kind_id) VALUES ('nala', 7, 2);
