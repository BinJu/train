CREATE TABLE team (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  token TEXT NOT NULL UNIQUE,
  desp TEXT
);

CREATE TABLE account (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  total INTEGER NOT NULL DEFAULT 0,
  in_stock INTEGER NOT NULL DEFAULT 0,
  data TEXT NOT NULL,
  owner INTEGER,
  desp TEXT,
  CONSTRAINT fk_team FOREIGN KEY(owner) REFERENCES team(id)
);

CREATE TABLE acnt_ctl (
  id SERIAL PRIMARY KEY,
  account_id INTEGER,
  team_id INTEGER,
  UNIQUE(account_id, team_id),
  CONSTRAINT fk_account FOREIGN KEY(account_id) REFERENCES account(id),
  CONSTRAINT fk_team FOREIGN KEY(team_id) REFERENCES team(id)
);

CREATE TABLE secret (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  data TEXT NOT NULL,
  owner INTEGER,
  desp TEXT,
  CONSTRAINT fk_team FOREIGN KEY(owner) REFERENCES team(id)
);

CREATE TABLE sec_ctl (
  id SERIAL PRIMARY KEY,
  secret_id INTEGER,
  team_id INTEGER,
  UNIQUE(secret_id, team_id),
  CONSTRAINT fk_secret FOREIGN KEY(secret_id) REFERENCES secret(id),
  CONSTRAINT fk_team FOREIGN KEY(team_id) REFERENCES team(id)
);

CREATE TABLE artifact (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  total INTEGER NOT NULL DEFAULT 0,
  target INTEGER NOT NULL DEFAULT 0,
  team_id INTEGER NOT NULL,
  build JSON NOT NULL,
  clean JSON,
  CONSTRAINT fk_team FOREIGN KEY(team_id) REFERENCES team(id)
);
