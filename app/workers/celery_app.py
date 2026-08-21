from celery import Celery

from app.config import settings

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
