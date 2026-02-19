from datetime import date

import config
from connectors.http_client import HTTPConnector
from connectors.sqlite_reader import SQLiteReader
from skills.base import BaseSkill


class DomainSkill(BaseSkill):
    name = "Domain Drop Hunter"
    description = "Get domain drop hunting results: matched domains, zone stats, disappeared domains"

    def __init__(self):
        self.http = HTTPConnector(config.DOMAIN_HUNTER_API)
        self.db = SQLiteReader(config.DOMAIN_HUNTER_DB)

    async def fetch_data(self, **params) -> dict:
        query_date = params.get("date", date.today().isoformat())
        min_score = params.get("min_score", 0)

        # Try HTTP first
        result = await self.http.get("/api/dashboard", {
            "date": query_date,
            "min_score": min_score,
        })
        if result:
            return {
                "source": "api",
                "matches": result.get("matches", [])[:20],
                "total_dropped": result.get("total_dropped", 0),
                "total_added": result.get("total_added", 0),
                "disappeared_total": result.get("disappeared_total", 0),
                "tlds": result.get("tlds", []),
                "date": query_date,
            }

        # Fallback to SQLite
        if not self.db.exists:
            return {"error": "Domain Drop Hunter database not found and API not running"}

        matches = self.db.query(
            "SELECT domain, tld, watchlist_term, score, match_type "
            "FROM matches WHERE drop_date = ? AND score >= ? "
            "ORDER BY score DESC LIMIT 20",
            (query_date, min_score),
        )
        stats = self.db.query(
            "SELECT tld, dropped, added FROM zone_stats WHERE stat_date = ?",
            (query_date,),
        )
        disappeared = self.db.query_one(
            "SELECT COUNT(*) as cnt FROM disappeared_domains WHERE status = 'pending'"
        )
        return {
            "source": "sqlite",
            "matches": matches,
            "total_dropped": sum(s.get("dropped", 0) for s in stats),
            "total_added": sum(s.get("added", 0) for s in stats),
            "disappeared_total": disappeared["cnt"] if disappeared else 0,
            "date": query_date,
        }
