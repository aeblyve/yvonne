CREATE TABLE IF NOT EXISTS item_map (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  item_id INTEGER,
  container_id INTEGER,
  quantity INTEGER,
  location TEXT,
  FOREIGN KEY(item_id) REFERENCES item(id),
  FOREIGN KEY(container_id) REFERENCES container(id)
)
