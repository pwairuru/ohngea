"""Generate sofia.conf.xml from DB sip_profiles + fallback.

Writes to /app/freeswitch_conf/autoload_configs/sofia.conf.xml (mounted to host's docker/freeswitch/conf)
and triggers FS reload via ESL.

If no DB profiles, generates default internal profile for LAN_IP (default 192.168.100.82) for test.example.com.
"""
import os
import pathlib
from lxml import etree

CONF_PATH = pathlib.Path("/app/freeswitch_conf/autoload_configs/sofia.conf.xml")
# Fallback host path if running outside docker (dev)
HOST_CONF = pathlib.Path("docker/freeswitch/conf/autoload_configs/sofia.conf.xml")


def _build_sofia_xml(profiles) -> bytes:
    root = etree.Element("configuration", name="sofia.conf", description="sofia Endpoint")
    profiles_el = etree.SubElement(root, "profiles")
    if not profiles:
        # fallback default as before
        lan_ip = os.environ.get("LAN_IP", "192.168.100.82")
        for name, port, ctx, realm in [
            ("internal", "5060", "test.example.com", "test.example.com"),
            ("internal-test", "5061", "test.example.com", "test.example.com"),
        ]:
            prof = etree.SubElement(profiles_el, "profile", name=name)
            settings = etree.SubElement(prof, "settings")
            etree.SubElement(settings, "param", name="sip-ip", value="0.0.0.0")
            etree.SubElement(settings, "param", name="ext-sip-ip", value=lan_ip)
            etree.SubElement(settings, "param", name="ext-rtp-ip", value=lan_ip)
            etree.SubElement(settings, "param", name="rtp-ip", value="0.0.0.0")
            etree.SubElement(settings, "param", name="sip-port", value=port)
            etree.SubElement(settings, "param", name="dialplan", value="XML")
            etree.SubElement(settings, "param", name="context", value=ctx)
            etree.SubElement(settings, "param", name="realm", value=realm)
            etree.SubElement(settings, "param", name="challenge-realm", value="auto_from")
            etree.SubElement(settings, "param", name="accept-blind-reg", value="false")
            etree.SubElement(settings, "param", name="inbound-late-negotiation", value="true")
            etree.SubElement(settings, "param", name="rtp-timer-name", value="soft")
            etree.SubElement(settings, "param", name="enable-100rel", value="false")
            etree.SubElement(settings, "param", name="disable-register", value="false")
            etree.SubElement(settings, "param", name="apply-inbound-acl", value="any")
            etree.SubElement(settings, "param", name="local-network-acl", value="localnet.auto")
    else:
        for p in profiles:
            name = getattr(p, "name", "internal")
            sip_port = str(getattr(p, "sip_port", 5060) or 5060)
            sip_ip = getattr(p, "sip_ip", None) or "0.0.0.0"
            ext_sip_ip = getattr(p, "ext_sip_ip", None) or os.environ.get("LAN_IP", "192.168.100.82")
            ext_rtp_ip = getattr(p, "ext_rtp_ip", None) or ext_sip_ip
            rtp_ip = getattr(p, "rtp_ip", None) or "0.0.0.0"
            realm = getattr(p, "realm", None) or getattr(p, "name", "test.example.com") or "test.example.com"
            context = getattr(p, "context", None) or realm
            # if realm looks like domain, use it, otherwise use ext_sip_ip
            if not realm:
                realm = ext_sip_ip
            if not context:
                context = realm
            prof = etree.SubElement(profiles_el, "profile", name=name)
            settings = etree.SubElement(prof, "settings")
            etree.SubElement(settings, "param", name="sip-ip", value=sip_ip)
            etree.SubElement(settings, "param", name="ext-sip-ip", value=ext_sip_ip)
            etree.SubElement(settings, "param", name="ext-rtp-ip", value=ext_rtp_ip)
            etree.SubElement(settings, "param", name="rtp-ip", value=rtp_ip)
            etree.SubElement(settings, "param", name="sip-port", value=sip_port)
            etree.SubElement(settings, "param", name="dialplan", value="XML")
            etree.SubElement(settings, "param", name="context", value=context)
            etree.SubElement(settings, "param", name="realm", value=realm)
            etree.SubElement(settings, "param", name="challenge-realm", value=getattr(p, "challenge_realm", None) or "auto_from")
            etree.SubElement(settings, "param", name="accept-blind-reg", value="false")
            etree.SubElement(settings, "param", name="inbound-late-negotiation", value="true")
            etree.SubElement(settings, "param", name="rtp-timer-name", value="soft")
            etree.SubElement(settings, "param", name="enable-100rel", value="false")
            etree.SubElement(settings, "param", name="disable-register", value="false")
            etree.SubElement(settings, "param", name="apply-inbound-acl", value=getattr(p, "apply_inbound_acl", None) or "any")
            etree.SubElement(settings, "param", name="local-network-acl", value=getattr(p, "local_network_acl", None) or "localnet.auto")
            if getattr(p, "hold_music", None):
                etree.SubElement(settings, "param", name="hold-music", value=p.hold_music)
            if getattr(p, "record_path", None):
                etree.SubElement(settings, "param", name="record-path", value=p.record_path)
    return etree.tostring(root, xml_declaration=True, encoding="UTF-8", pretty_print=True)


async def regenerate_sofia_conf(session=None):
    """Regenerate sofia.conf.xml from DB and reload FS."""
    profiles = []
    try:
        if session is not None:
            from sqlalchemy import select
            from app.sip_profiles.models import SipProfile

            r = await session.execute(select(SipProfile).where(SipProfile.enabled == True).order_by(SipProfile.name))
            profiles = r.scalars().all()
        else:
            # try to get session
            from app.db import SessionLocal
            from sqlalchemy import select
            from app.sip_profiles.models import SipProfile

            async with SessionLocal() as s:
                r = await s.execute(select(SipProfile).where(SipProfile.enabled == True).order_by(SipProfile.name))
                profiles = r.scalars().all()
    except Exception:
        profiles = []

    xml = _build_sofia_xml(profiles)

    # Write to both possible paths
    written = False
    for p in [CONF_PATH, HOST_CONF]:
        try:
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(xml)
            written = True
        except Exception:
            continue
    if not written:
        # fallback to direct host path
        try:
            pathlib.Path("docker/freeswitch/conf/autoload_configs/sofia.conf.xml").write_bytes(xml)
        except Exception:
            pass

    # Reload FS
    try:
        from app.freeswitch.esl_client import esl_client

        await esl_client.reloadxml()
        # also restart profiles
        for prof in profiles:
            try:
                await esl_client.api(f"sofia profile {prof.name} restart")
            except Exception:
                pass
        # also restart default internal if no DB profiles
        if not profiles:
            for name in ["internal", "internal-test"]:
                try:
                    await esl_client.api(f"sofia profile {name} restart")
                except Exception:
                    pass
    except Exception:
        pass
    return xml
