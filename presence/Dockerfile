FROM python:3.12-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY server.py __init__.py __main__.py ./presence/

ENV PYTHONUNBUFFERED=1

CMD ["python", "-m", "presence"]
