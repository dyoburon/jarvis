from abc import ABC, abstractmethod


class BaseSkill(ABC):
    """Base class for Jarvis skills."""

    name: str
    description: str

    @abstractmethod
    async def fetch_data(self, **params) -> dict:
        """Fetch raw data from the connected project."""
        ...

    def format_prompt(self, data: dict, user_query: str) -> str:
        """Build a prompt for Gemini to summarize the data."""
        import json
        return (
            f"You are Jarvis, reporting on {self.name} data.\n"
            f"User asked: \"{user_query}\"\n\n"
            f"Data:\n{json.dumps(data, indent=2, default=str)}\n\n"
            "Provide a concise summary (3-5 sentences). Be specific with numbers. "
            "Format for readability with bullet points where appropriate."
        )
