from pydantic import AliasChoices, Field, model_validator
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Ohngea PBX — 12-factor config.

    All fields have sane defaults for local dev; override via .env or env vars.
    See .env.example for documentation.
    Valkey is primary cache/broker; REDIS_URL kept for backwards compat and celery.
    """

    database_url: str = "postgresql+asyncpg://ohngea:ohngea@localhost:5432/ohngea"
    # Primary cache URL (valkey). Supports valkey://, valkeys://, redis://, rediss:// for migration.
    valkey_url: str = Field(
        default="valkey://localhost:6379/0",
        validation_alias=AliasChoices("VALKEY_URL", "REDIS_URL"),
    )
    # Deprecated alias — celery/kombu still require redis:// scheme, so keep separate field.
    # This will be derived from valkey_url if only VALKEY_URL is set.
    redis_url: str = Field(
        default="redis://localhost:6379/0",
        validation_alias=AliasChoices("REDIS_URL"),
    )
    jwt_secret: str = "dev-secret-change"
    jwt_algorithm: str = "HS256"
    jwt_expire_minutes: int = 60
    # CORS — explicit origins in prod. Comma-separated via CORS_ORIGINS env.
    # Default allows local dev (vite 5173, 3000). Never use "*" with credentials.
    cors_origins: str = "http://localhost:3000,http://localhost:5173"
    env: str = Field(default="dev", validation_alias=AliasChoices("ENV", "APP_ENV"))
    fs_esl_host: str = "127.0.0.1"
    fs_esl_port: int = 8021
    fs_esl_password: str = "ClueCon"
    fs_xml_user: str = "freeswitch"
    fs_xml_password: str = "ClueCon"

    # SMTP for email_queue worker
    smtp_host: str = Field(default="localhost", validation_alias=AliasChoices("SMTP_HOST", "MAIL_HOST"))
    smtp_port: int = Field(default=1025, validation_alias=AliasChoices("SMTP_PORT", "MAIL_PORT"))
    smtp_user: str | None = Field(default=None, validation_alias=AliasChoices("SMTP_USER", "MAIL_USER"))
    smtp_password: str | None = Field(default=None, validation_alias=AliasChoices("SMTP_PASSWORD", "MAIL_PASSWORD"))
    smtp_use_tls: bool = Field(default=False, validation_alias=AliasChoices("SMTP_USE_TLS", "MAIL_TLS"))
    smtp_from: str = Field(default="noreply@ohngea.local", validation_alias=AliasChoices("SMTP_FROM", "MAIL_FROM"))
    # S3 / MinIO for recordings object store, partitioned by domain_id -> recordings/{domain_id}/...
    s3_endpoint: str | None = Field(default=None, validation_alias=AliasChoices("S3_ENDPOINT", "MINIO_ENDPOINT"))
    s3_bucket: str = Field(default="ohngea-recordings", validation_alias=AliasChoices("S3_BUCKET", "MINIO_BUCKET"))
    s3_access_key: str | None = Field(default=None, validation_alias=AliasChoices("S3_ACCESS_KEY", "MINIO_ACCESS_KEY"))
    s3_secret_key: str | None = Field(default=None, validation_alias=AliasChoices("S3_SECRET_KEY", "MINIO_SECRET_KEY"))
    s3_region: str = Field(default="us-east-1", validation_alias=AliasChoices("S3_REGION", "MINIO_REGION"))
    s3_use_ssl: bool = Field(default=False, validation_alias=AliasChoices("S3_USE_SSL", "MINIO_USE_SSL"))
    s3_presign_ttl: int = Field(default=3600, validation_alias=AliasChoices("S3_PRESIGN_TTL"))
    # Legacy recording base dir fallback (when S3 not configured)
    recordings_base_dir: str = Field(default="/var/lib/freeswitch/recordings")

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
        case_sensitive=False,
    )

    @model_validator(mode="after")
    def _check_prod_secrets(self):
        """Fail fast in prod with default secrets."""
        if self.env.lower() in ("prod", "production"):
            bad = []
            if self.jwt_secret == "dev-secret-change":
                bad.append("JWT_SECRET must be set in prod")
            if self.fs_esl_password == "ClueCon":
                bad.append("FS_ESL_PASSWORD must be changed in prod")
            if self.fs_xml_password == "ClueCon":
                bad.append("FS_XML_PASSWORD must be changed in prod")
            if bad:
                raise ValueError("; ".join(bad))
        return self

    @model_validator(mode="after")
    def _sync_valkey_redis_urls(self):  # type: ignore[no-redef]
        """Sync valkey/redis URLs for backwards compat.

        - If only REDIS_URL was set (valkey_url still default via REDIS_URL alias fallback),
          keep redis_url as source, but ensure valkey_url reflects same host (supports redis://).
        - If only VALKEY_URL was set (redis_url still default), derive redis_url with redis:// scheme
          for celery/kombu which only understands redis://.
        """
        default_valkey = "valkey://localhost:6379/0"
        default_redis = "redis://localhost:6379/0"
        # When both were explicitly set (docker-compose sets both), do nothing.
        # When valkey is default but redis is customized, valkey already got REDIS_URL via alias,
        # so no action needed — but handle case where alias didn't populate due to validation_alias ordering.
        if self.valkey_url == default_valkey and self.redis_url != default_redis:
            # REDIS_URL set, VALKEY_URL not — copy (valkey-py supports redis://)
            self.valkey_url = self.redis_url
        elif self.redis_url == default_redis and self.valkey_url != default_valkey:
            # VALKEY_URL set, REDIS_URL not — derive redis:// for celery
            if self.valkey_url.startswith("valkey://"):
                self.redis_url = self.valkey_url.replace("valkey://", "redis://", 1)
            elif self.valkey_url.startswith("valkeys://"):
                self.redis_url = self.valkey_url.replace("valkeys://", "rediss://", 1)
            else:
                self.redis_url = self.valkey_url
        return self


settings = Settings()
