#!/bin/bash
# jarvis start — boots up Jarvis
cd "$(dirname "$0")"
source .venv/bin/activate
python main.py
