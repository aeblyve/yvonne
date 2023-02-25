CREATE TABLE IF NOT EXISTS item_location (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  item_id INTEGER NOT NULL,
  container_id INTEGER NOT NULL,
  quantity INTEGER,
  FOREIGN KEY(item_id) REFERENCES item(id),
  FOREIGN KEY(container_id) REFERENCES container(id)
)
