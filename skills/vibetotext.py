import config
from connectors.sqlite_reader import SQLiteReader
from skills.base import BaseSkill


class VibeToTextSkill(BaseSkill):
    name = "VibeToText"
    description = "Get voice transcription statistics: total words dictated, sessions, WPM"

    def __init__(self):
        self.db = SQLiteReader(config.VIBETOTEXT_DB)

    async def fetch_data(self, **params) -> dict:
        if not self.db.exists:
            return {"error": "VibeToText database not found"}

        limit = params.get("limit", 10)

        stats = self.db.query_one(
            "SELECT COUNT(*) as sessions, "
            "SUM(word_count) as total_words, "
            "AVG(wpm) as avg_wpm, "
            "SUM(duration_seconds) as total_seconds, "
            "AVG(sentiment) as avg_sentiment "
            "FROM entries"
        )
        recent = self.db.query(
            "SELECT text, mode, timestamp, word_count, wpm, sentiment "
            "FROM entries ORDER BY timestamp DESC LIMIT ?",
            (limit,),
        )
        return {
            "source": "sqlite",
            "stats": stats,
            "recent_entries": recent,
        }
