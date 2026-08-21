from uuid import UUID
from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession


async def ensure_tenant_partition(session: AsyncSession, domain_id: UUID):
    """For HASH partitioning, no per-tenant creation needed. Validate pruning works.
    
    For LIST strategy, would create partition here. Keeping as no-op for HASH.
    """
    # Check explain to ensure pruning would work — no-op
    # Could log
    pass


async def explain_partition_pruning(session: AsyncSession, domain_id: UUID) -> str:
    result = await session.execute(text("EXPLAIN SELECT * FROM cdr WHERE domain_id = :id"), {"id": str(domain_id)})
    rows = result.fetchall()
    return "\n".join([r[0] for r in rows])
