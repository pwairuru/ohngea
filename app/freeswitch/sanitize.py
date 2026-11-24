"""ESL argument sanitization — prevent command injection."""
from __future__ import annotations

import re
from uuid import UUID

_DIAL_RE = re.compile(r"^[0-9a-zA-Z+@._\-#*]+$")
_PATH_RE = re.compile(r"^[0-9a-zA-Z_\-./]+$")

def sanitize_dial(s: str, max_len: int = 64) -> str:
    s = (s or "").strip()[:max_len]
    if not s or not _DIAL_RE.match(s):
        raise ValueError("invalid dial string")
    if any(x in s for x in (";", "&", "|", "$", "`", "(", ")", "\n", "\r")):
        raise ValueError("invalid dial string")
    return s

def sanitize_uuid(uid: str) -> str:
    return str(UUID(uid))

def sanitize_path(p: str, max_len: int = 256) -> str:
    p = (p or "").strip()[:max_len]
    if not p or not _PATH_RE.match(p):
        raise ValueError("invalid path")
    if ".." in p or any(x in p for x in (";", "&", "|", "$", "`", "(", ")", "\n", "\r")):
        raise ValueError("invalid path")
    return p
