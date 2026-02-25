"""
Configuration module for Jarvis.

Provides YAML-based configuration with Pydantic validation.
"""

from .loader import load_config, get_config_path
from .schema import JarvisConfig

__all__ = ["load_config", "get_config_path", "JarvisConfig"]
