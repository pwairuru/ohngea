# Support valkey:// -> redis:// for celery/kombu which still requires redis scheme.
# valkey server is protocol-compatible, so redis://valkey:6379/0 works against valkey.
# Also register valkey transport alias if kombu supports it (future-proof).
try:
    from kombu.transport import TRANSPORT_ALIASES

    TRANSPORT_ALIASES.setdefault("valkey", "kombu.transport.redis:Transport")
    TRANSPORT_ALIASES.setdefault("valkeys", "kombu.transport.redis:Transport")
except Exception:
    pass

from celery import Celery

from app.config import settings

# Celery broker/backend must use redis:// scheme (kombu only understands redis).
# settings.redis_url is derived from valkey_url but forced to redis:// scheme.
celery_app = Celery(
    "ohngea",
    broker=settings.redis_url,
    backend=settings.redis_url,
)

celery_app.conf.update(
    task_serializer="json",
    accept_content=["json"],
    result_serializer="json",
    timezone="UTC",
    enable_utc=True,
)


@celery_app.task
def ping():
    return "pong"


@celery_app.task
def send_email(to: str, subject: str, body: str):
    # placeholder — integrate SMTP later
    return {"to": to, "subject": subject, "queued": True}
