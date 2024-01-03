-- Your SQL goes here
CREATE TABLE candidates (
  id TEXT PRIMARY KEY NOT NULL,
  label TEXT NOT NULL,
  voter BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE criteria (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  name TEXT NOT NULL,
  min SMALLINT NOT NULL,
  max SMaLLINT NOT NULL,
  weight FLOAT NULLABLE
);

CREATE TABLE ballots (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  human_identifier TEXT NOT NULL,
  voter TEXT NOT NULL,
  candidate TEXT NOT NULL,
  sum SMALLINT NOT NULL,
  weighted FLOAT NOT NULL,
  mean FLOAT NOT NULL,
  notes TEXT NOT NULL,
  votes TEXT T NULL,
  voted_on TIMESTAMP NOT NULL
);
CREATE TABLE votings (
    name TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL,
    candidates TEXT NOT NULL,
    categories TEXT NOT NULL,
    styles TEXT NOT NULL,
    invite_code TEXT NOT NULL
);
