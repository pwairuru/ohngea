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
from app.domains.models import Domain, DomainSettings, DomainPlugin, DomainApp  # noqa
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
from app.ring_groups.models import RingGroup, RingGroupMember  # noqa
from app.time_conditions.models import TimeCondition  # noqa
from app.voicemails.models import Voicemail, VoicemailDestination  # noqa
from app.conferences.models import Conference, ConferenceUser  # noqa
from app.fax.models import Fax  # noqa
from app.call_flows.models import CallFlow  # noqa
from app.music_on_hold.models import MusicOnHold  # noqa
from app.follow_me.models import FollowMe, FollowMeDestination  # noqa
from app.call_block.models import CallBlock  # noqa
from app.destinations.models import Destination  # noqa
from app.pin_numbers.models import PinNumber  # noqa
from app.phrases.models import Phrase, PhraseDetail  # noqa
from app.streams.models import Stream  # noqa
from app.bridges.models import Bridge  # noqa
from app.fifo.models import Fifo  # noqa
from app.call_centers.models import CallCenter, CallCenterAgent  # noqa
from app.vars.models import Var  # noqa
from app.number_translations.models import NumberTranslation  # noqa
from app.conference_profiles.models import ConferenceProfile, ConferenceProfileParam  # noqa
from app.conference_controls.models import ConferenceControl, ConferenceControlDetail  # noqa
from app.conference_centers.models import ConferenceCenter  # noqa
from app.access_controls.models import AccessControl, AccessControlNode  # noqa
from app.call_broadcast.models import CallBroadcast  # noqa
from app.emergency_logs.models import EmergencyLog  # noqa
from app.voicemail_greetings.models import VoicemailGreeting  # noqa
from app.fax_queue.models import FaxQueue  # noqa
from app.extension_settings.models import ExtensionSetting  # noqa
from app.email_queue.models import EmailQueue  # noqa
from app.sofia_global_settings.models import SofiaGlobalSetting  # noqa
from app.feature_codes.models import FeatureCode  # noqa
from app.database_transactions.models import DatabaseTransaction  # noqa
from app.event_guard.models import EventGuardLog  # noqa
from app.modules.models import Module  # noqa
from app.webrtc_settings.models import WebRTCSettings  # noqa

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
