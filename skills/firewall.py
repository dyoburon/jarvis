import config
from connectors.http_client import HTTPConnector
from skills.base import BaseSkill


class FirewallSkill(BaseSkill):
    name = "Great Firewall"
    description = "Get chat moderation stats: recent messages, blocked count, superchat activity"

    def __init__(self):
        self.http = HTTPConnector(config.FIREWALL_API)

    async def fetch_data(self, **params) -> dict:
        # All data is in-memory on the server — HTTP only
        messages = await self.http.get("/api/messages")
        approved = await self.http.get("/stream/messages")

        if messages is None and approved is None:
            return {"error": "Great Firewall is not running"}

        return {
            "source": "api",
            "recent_messages": (messages or [])[-10:],
            "total_filtered": len(messages) if messages else 0,
            "recent_approved": (approved or [])[-5:],
            "total_approved": len(approved) if approved else 0,
        }
