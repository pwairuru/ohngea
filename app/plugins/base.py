from typing import Protocol

class PluginBase(Protocol):
    name: str
    version: str
    license: str
    description: str
