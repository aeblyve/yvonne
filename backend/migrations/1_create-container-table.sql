CREATE TABLE IF NOT EXISTS container (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  parent_container_id INTEGER,
  name TEXT NOT NULL UNIQUE,
  note TEXT,
  photo BLOB,
  FOREIGN KEY(parent_container_id) REFERENCES container(id) ON DELETE CASCADE
);
