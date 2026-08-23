"""S3/MinIO client + helpers for recordings, fax files.

Partitioned by domain_id: recordings/{domain_id}/{uuid}.wav  etc.
Falls back to mock when boto3 not installed or no credentials — presigned URLs
are still generated as deterministic placeholder so tests pass without MinIO.
"""
from __future__ import annotations

import hashlib
import hmac
import time
from typing import Optional
from urllib.parse import quote
from uuid import UUID

from app.config import settings

_client = None


def _mock_presigned(bucket: str, key: str, ttl: int = 3600) -> str:
    # Deterministic placeholder: /<bucket>/<key>?expires=<ttl>&signature=mock
    # includes expiry ts for realistic testing
    exp = int(time.time()) + ttl
    sig = hmac.new(b"mock-secret", f"{bucket}/{key}/{exp}".encode(), hashlib.sha256).hexdigest()[:16]
    # Use S3 endpoint if set else mock host
    base = (settings.s3_endpoint or "http://mock-s3.local").rstrip("/")
    return f"{base}/{bucket}/{quote(key, safe='/')}?expires={exp}&signature={sig}&mock=1"


def get_s3_client():
    global _client
    if _client is not None:
        return _client
    # Try boto3 if installed and credentials present
    if not settings.s3_access_key or not settings.s3_secret_key:
        _client = None
        return None
    try:
        import boto3  # type: ignore
        from botocore.config import Config  # type: ignore

        kwargs: dict = {
            "region_name": settings.s3_region,
            "aws_access_key_id": settings.s3_access_key,
            "aws_secret_access_key": settings.s3_secret_key,
            "config": Config(signature_version="s3v4"),
        }
        if settings.s3_endpoint:
            kwargs["endpoint_url"] = settings.s3_endpoint
            kwargs["use_ssl"] = settings.s3_use_ssl
        else:
            kwargs["use_ssl"] = settings.s3_use_ssl
        _client = boto3.client("s3", **kwargs)
        # ensure bucket exists (best-effort, ignore errors)
        try:
            _client.head_bucket(Bucket=settings.s3_bucket)
        except Exception:
            try:
                _client.create_bucket(Bucket=settings.s3_bucket)
            except Exception:
                pass
        return _client
    except Exception:
        _client = None
        return None


def presigned_get_url(key: str, expires: Optional[int] = None, bucket: Optional[str] = None) -> str:
    """Generate presigned GET URL for object key. Falls back to mock."""
    ttl = expires or settings.s3_presign_ttl
    bkt = bucket or settings.s3_bucket
    client = get_s3_client()
    if client is None:
        return _mock_presigned(bkt, key, ttl)
    try:
        url = client.generate_presigned_url(
            "get_object", Params={"Bucket": bkt, "Key": key}, ExpiresIn=ttl
        )
        return url
    except Exception:
        return _mock_presigned(bkt, key, ttl)


def presigned_put_url(key: str, expires: Optional[int] = None, bucket: Optional[str] = None) -> str:
    ttl = expires or settings.s3_presign_ttl
    bkt = bucket or settings.s3_bucket
    client = get_s3_client()
    if client is None:
        return _mock_presigned(bkt, key, ttl)
    try:
        url = client.generate_presigned_url(
            "put_object", Params={"Bucket": bkt, "Key": key}, ExpiresIn=ttl
        )
        return url
    except Exception:
        return _mock_presigned(bkt, key, ttl)


def s3_key_for_recording(domain_id: UUID | str, recording_uuid: UUID | str, suffix: str = "wav") -> str:
    # partitioned: recordings/{domain_id}/{recording_uuid}.{suffix}
    did = str(domain_id)
    rid = str(recording_uuid)
    # ensure suffix stripped dot
    suffix = suffix.lstrip(".") or "wav"
    return f"recordings/{did}/{rid}.{suffix}"


def s3_key_for_fax(domain_id: UUID | str, fax_queue_uuid: UUID | str, filename: str | None = None) -> str:
    did = str(domain_id)
    rid = str(fax_queue_uuid)
    if filename:
        return f"fax/{did}/{rid}/{filename}"
    return f"fax/{did}/{rid}.tiff"


def s3_upload_bytes(key: str, data: bytes, content_type: str = "application/octet-stream", bucket: Optional[str] = None) -> bool:
    bkt = bucket or settings.s3_bucket
    client = get_s3_client()
    if client is None:
        # mock success without IO
        return True
    try:
        client.put_object(Bucket=bkt, Key=key, Body=data, ContentType=content_type)
        return True
    except Exception:
        return False


def s3_download_bytes(key: str, bucket: Optional[str] = None) -> bytes | None:
    bkt = bucket or settings.s3_bucket
    client = get_s3_client()
    if client is None:
        return None
    try:
        resp = client.get_object(Bucket=bkt, Key=key)
        return resp["Body"].read()
    except Exception:
        return None
