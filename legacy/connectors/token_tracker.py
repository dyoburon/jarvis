import sqlite3
from datetime import datetime
from pathlib import Path


class TokenTracker:
    """Tracks cumulative Gemini API token usage in SQLite."""

    def __init__(self, db_path: str | Path):
        self.db_path = str(db_path)
        self._init_db()

    def _init_db(self):
        Path(self.db_path).parent.mkdir(parents=True, exist_ok=True)
        conn = sqlite3.connect(self.db_path)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS token_usage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                model TEXT NOT NULL,
                session_type TEXT NOT NULL,
                prompt_tokens INTEGER NOT NULL DEFAULT 0,
                completion_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0
            )
        """)
        conn.commit()
        conn.close()

    def record(self, model: str, session_type: str,
               prompt_tokens: int, completion_tokens: int,
               total_tokens: int):
        conn = sqlite3.connect(self.db_path)
        conn.execute(
            """INSERT INTO token_usage
               (timestamp, model, session_type, prompt_tokens, completion_tokens, total_tokens)
               VALUES (?, ?, ?, ?, ?, ?)""",
            (datetime.now().isoformat(), model, session_type,
             prompt_tokens, completion_tokens, total_tokens),
        )
        conn.commit()
        conn.close()

    def get_totals(self) -> dict:
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        row = conn.execute("""
            SELECT
                COUNT(*) as total_calls,
                COALESCE(SUM(prompt_tokens), 0) as total_prompt,
                COALESCE(SUM(completion_tokens), 0) as total_completion,
                COALESCE(SUM(total_tokens), 0) as total_tokens
            FROM token_usage
        """).fetchone()
        conn.close()
        return dict(row)

    def get_by_session_type(self) -> list[dict]:
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        rows = conn.execute("""
            SELECT
                session_type,
                COUNT(*) as calls,
                SUM(prompt_tokens) as prompt_tokens,
                SUM(completion_tokens) as completion_tokens,
                SUM(total_tokens) as total_tokens
            FROM token_usage
            GROUP BY session_type
        """).fetchall()
        conn.close()
        return [dict(r) for r in rows]

    def get_by_model(self) -> list[dict]:
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        rows = conn.execute("""
            SELECT
                model,
                COUNT(*) as calls,
                SUM(prompt_tokens) as prompt_tokens,
                SUM(completion_tokens) as completion_tokens,
                SUM(total_tokens) as total_tokens
            FROM token_usage
            GROUP BY model
        """).fetchall()
        conn.close()
        return [dict(r) for r in rows]

    def get_recent(self, limit: int = 20) -> list[dict]:
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        rows = conn.execute(
            "SELECT * FROM token_usage ORDER BY id DESC LIMIT ?",
            (limit,),
        ).fetchall()
        conn.close()
        return [dict(r) for r in rows]
