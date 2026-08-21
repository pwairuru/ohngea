from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Application settings — 12-factor via environment.

    All fields have sane defaults for local dev; override via .env or env vars.
    See .env.example for documentation.
    """

    database_url: str = "postgresql+asyncpg://ohngea:ohngea@localhost:5432/ohngea"
    redis_url: str = "redis://localhost:6379/0"
    jwt_secret: str = "dev-secret-change"
    jwt_algorithm: str = "HS256"
    jwt_expire_minutes: int = 60 * 24
    fs_esl_host: str = "127.0.0.1"
    fs_esl_port: int = 8021
    fs_esl_password: str = "ClueCon"
    fs_xml_user: str = "freeswitch"
    fs_xml_password: str = "ClueCon"

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
        case_sensitive=False,
    )


settings = Settings()
