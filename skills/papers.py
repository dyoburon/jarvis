from datetime import date

import config
from connectors.http_client import HTTPConnector
from connectors.sqlite_reader import SQLiteReader
from skills.base import BaseSkill


class PaperSkill(BaseSkill):
    name = "Paper Feeder"
    description = "Get research paper matches from arXiv, grouped by interest area"

    def __init__(self):
        self.http = HTTPConnector(config.PAPER_FEEDER_API)
        self.db = SQLiteReader(config.PAPER_FEEDER_DB)

    async def fetch_data(self, **params) -> dict:
        query_date = params.get("date", date.today().isoformat())
        interest_filter = params.get("interest")

        # Try HTTP first
        result = await self.http.get("/api/dashboard", {"date": query_date})
        if result:
            papers = result.get("papers_by_interest", {})
            if interest_filter:
                papers = {k: v for k, v in papers.items()
                          if interest_filter.lower() in k.lower()}
            return {
                "source": "api",
                "papers_by_interest": {k: v[:5] for k, v in papers.items()},
                "total_papers": result.get("total_papers", 0),
                "total_matches": result.get("total_matches", 0),
                "active_interests": result.get("active_interests", 0),
                "date": query_date,
            }

        # Fallback to SQLite
        if not self.db.exists:
            return {"error": "Paper Feeder database not found and API not running"}

        if query_date == date.today().isoformat():
            has_data = self.db.query_one("SELECT 1 FROM paper_matches WHERE match_date = ?", (query_date,))
            if not has_data:
                latest = self.db.query_one("SELECT MAX(match_date) as md FROM paper_matches")
                if latest and latest["md"]:
                    query_date = latest["md"]

        query = (
            "SELECT p.title, p.primary_category, pm.score, i.name as interest "
            "FROM paper_matches pm "
            "JOIN papers p ON p.id = pm.paper_id "
            "JOIN interests i ON i.id = pm.interest_id "
            "WHERE pm.match_date = ? "
        )
        params_list = [query_date]

        if interest_filter:
            query += "AND i.name LIKE ? "
            params_list.append(f"%{interest_filter}%")
        query += "ORDER BY pm.score DESC"

        papers = self.db.query(query, tuple(params_list))

        papers_by_interest = {}
        for p in papers:
            interest = p["interest"]
            if interest not in papers_by_interest:
                papers_by_interest[interest] = []
            if len(papers_by_interest[interest]) < 5:
                papers_by_interest[interest].append(p)

        return {
            "source": "sqlite",
            "papers_by_interest": papers_by_interest,
            "date": query_date,
        }
