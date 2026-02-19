import sqlite3
from pathlib import Path


class SQLiteReader:
    """Read-only SQLite access for data connectors."""

    def __init__(self, db_path: str | Path):
        self.db_path = str(db_path)

    @property
    def exists(self) -> bool:
        return Path(self.db_path).exists()

    def query(self, sql: str, params: tuple = ()) -> list[dict]:
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        rows = conn.execute(sql, params).fetchall()
        conn.close()
        return [dict(r) for r in rows]

    def query_one(self, sql: str, params: tuple = ()) -> dict | None:
        results = self.query(sql, params)
        return results[0] if results else None
