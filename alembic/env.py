import asyncio
from logging.config import fileConfig

from alembic import context
from sqlalchemy import pool
from sqlalchemy.engine import Connection
from sqlalchemy.ext.asyncio import async_engine_from_config
from sqlmodel import SQLModel

from app.config import settings

config = context.config

if config.config_file_name is not None:
    fileConfig(config.config_file_name)

# Override sqlalchemy.url from settings (12-factor)
config.set_main_option("sqlalchemy.url", settings.database_url)

# Import all models to populate SQLModel.metadata
from app.domains.models import Domain, DomainSettings, DomainPlugin  # noqa
from app.auth.models import User, Group, UserGroup, Permission, GroupPermission  # noqa
from app.extensions.models import Extension  # noqa
from app.dialplans.models import Dialplan, DialplanDetail  # noqa
from app.gateways.models import Gateway  # noqa
from app.sip_profiles.models import SipProfile  # noqa
from app.ivr_menus.models import IvrMenu, IvrMenuOption  # noqa
from app.devices.models import Device  # noqa
from app.cdr.models import Cdr  # noqa
from app.recordings.models import Recording  # noqa
from app.contacts.models import Contact  # noqa

target_metadata = SQLModel.metadata


def run_migrations_offline() -> None:
    url = config.get_main_option("sqlalchemy.url")
    context.configure(
        url=url,
        target_metadata=target_metadata,
        literal_binds=True,
        dialect_opts={"paramstyle": "named"},
    )
    with context.begin_transaction():
        context.run_migrations()


def do_run_migrations(connection: Connection) -> None:
    context.configure(connection=connection, target_metadata=target_metadata)
    with context.begin_transaction():
        context.run_migrations()


async def run_async_migrations() -> None:
    connectable = async_engine_from_config(
        config.get_section(config.config_ini_section, {}),
        prefix="sqlalchemy.",
        poolclass=pool.NullPool,
    )
    async with connectable.connect() as connection:
        await connection.run_sync(do_run_migrations)
    await connectable.dispose()


def run_migrations_online() -> None:
    asyncio.run(run_async_migrations())


if context.is_offline_mode():
    run_migrations_offline()
else:
    run_migrations_online()
