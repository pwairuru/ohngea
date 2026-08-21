from fastapi import FastAPI

app = FastAPI(title="Ohngea PBX", version="0.1.0")


@app.get("/health", tags=["health"])
async def health() -> dict[str, str]:
    """Liveness probe — no DB check, always 200 when process is up."""
    return {"status": "ok"}
