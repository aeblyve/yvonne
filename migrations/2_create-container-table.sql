CREATE TABLE IF NOT EXISTS container (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  site_id INTEGER,
  name TEXT NOT NULL UNIQUE,
  note TEXT,
  photo BLOB,
  FOREIGN KEY(site_id) REFERENCES site(id)
);