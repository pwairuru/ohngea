"""Operational fixtures — ACL, conference profiles, music on hold."""

from __future__ import annotations

ACCESS_CONTROLS: list[dict] = [
    {
        "name": "rfc1918",
        "default": "deny",
        "description": "Private networks",
        "nodes": [
            {"type": "allow", "cidr": "10.0.0.0/8", "description": "RFC1918"},
            {"type": "allow", "cidr": "172.16.0.0/12", "description": "RFC1918"},
            {"type": "allow", "cidr": "192.168.0.0/16", "description": "RFC1918"},
        ],
    },
    {
        "name": "providers",
        "default": "deny",
        "description": "SIP providers (add nodes per trunk)",
        "nodes": [],
    },
]

CONFERENCE_PROFILES: list[dict] = [
    {
        "name": "default",
        "params": [
            {"name": "cdr-log-dir", "value": "auto"},
            {"name": "domain", "value": ""},
            {"name": "rate", "value": "8000"},
            {"name": "interval", "value": "20"},
            {"name": "energy-level", "value": "15"},
            {"name": "auto-gain-level", "value": "0"},
            {"name": "caller-controls", "value": "default"},
            {"name": "moderator-controls", "value": "moderator"},
            {"name": "muted-sound", "value": "conference/conf-muted.wav"},
            {"name": "unmuted-sound", "value": "conference/conf-unmuted.wav"},
            {"name": "alone-sound", "value": "conference/conf-alone.wav"},
            {"name": "moh-sound", "value": "local_stream://default"},
            {"name": "enter-sound", "value": "tone_stream://%(200,0,500,600,700)"},
            {"name": "exit-sound", "value": "tone_stream://%(500,0,300,200,100,50,25)"},
            {"name": "kicked-sound", "value": "conference/conf-kicked.wav"},
            {"name": "locked-sound", "value": "conference/conf-locked.wav"},
            {"name": "is-locked-sound", "value": "conference/conf-is-locked.wav"},
            {"name": "is-unlocked-sound", "value": "conference/conf-is-unlocked.wav"},
            {"name": "pin-sound", "value": "conference/conf-pin.wav"},
            {"name": "bad-pin-sound", "value": "conference/conf-bad-pin.wav"},
            {"name": "caller-id-name", "value": ""},
            {"name": "caller-id-number", "value": ""},
            {"name": "comfort-noise", "value": "true"},
        ],
    },
    {
        "name": "wait-mod",
        "params": [
            {"name": "cdr-log-dir", "value": "auto"},
            {"name": "conference-flags", "value": "wait-mod"},
            {"name": "domain", "value": ""},
            {"name": "rate", "value": "8000"},
            {"name": "interval", "value": "20"},
            {"name": "energy-level", "value": "15"},
            {"name": "auto-gain-level", "value": "0"},
            {"name": "caller-controls", "value": "default"},
            {"name": "moderator-controls", "value": "moderator"},
            {"name": "muted-sound", "value": "conference/conf-muted.wav"},
            {"name": "unmuted-sound", "value": "conference/conf-unmuted.wav"},
            {"name": "alone-sound", "value": "conference/conf-alone.wav"},
            {"name": "moh-sound", "value": "local_stream://default"},
            {"name": "enter-sound", "value": "tone_stream://%(200,0,500,600,700)"},
            {"name": "exit-sound", "value": "tone_stream://%(500,0,300,200,100,50,25)"},
            {"name": "kicked-sound", "value": "conference/conf-kicked.wav"},
            {"name": "locked-sound", "value": "conference/conf-locked.wav"},
            {"name": "is-locked-sound", "value": "conference/conf-is-locked.wav"},
            {"name": "is-unlocked-sound", "value": "conference/conf-is-unlocked.wav"},
            {"name": "pin-sound", "value": "conference/conf-pin.wav"},
            {"name": "bad-pin-sound", "value": "conference/conf-bad-pin.wav"},
            {"name": "caller-id-name", "value": ""},
            {"name": "caller-id-number", "value": ""},
            {"name": "comfort-noise", "value": "true"},
        ],
    },
    {
        "name": "wideband",
        "params": [
            {"name": "domain", "value": ""},
            {"name": "rate", "value": "16000"},
            {"name": "interval", "value": "20"},
            {"name": "energy-level", "value": "15"},
            {"name": "auto-gain-level", "value": "0"},
            {"name": "caller-controls", "value": "default"},
            {"name": "moderator-controls", "value": "moderator"},
            {"name": "muted-sound", "value": "conference/conf-muted.wav"},
            {"name": "unmuted-sound", "value": "conference/conf-unmuted.wav"},
            {"name": "alone-sound", "value": "conference/conf-alone.wav"},
            {"name": "moh-sound", "value": "local_stream://default"},
            {"name": "enter-sound", "value": "tone_stream://%(200,0,500,600,700)"},
            {"name": "exit-sound", "value": "tone_stream://%(500,0,300,200,100,50,25)"},
            {"name": "kicked-sound", "value": "conference/conf-kicked.wav"},
            {"name": "locked-sound", "value": "conference/conf-locked.wav"},
            {"name": "is-locked-sound", "value": "conference/conf-is-locked.wav"},
            {"name": "is-unlocked-sound", "value": "conference/conf-is-unlocked.wav"},
            {"name": "pin-sound", "value": "conference/conf-pin.wav"},
            {"name": "bad-pin-sound", "value": "conference/conf-bad-pin.wav"},
            {"name": "caller-id-name", "value": ""},
            {"name": "caller-id-number", "value": ""},
            {"name": "comfort-noise", "value": "true"},
        ],
    },
    {
        "name": "ultrawideband",
        "params": [
            {"name": "domain", "value": ""},
            {"name": "rate", "value": "32000"},
            {"name": "interval", "value": "20"},
            {"name": "energy-level", "value": "15"},
            {"name": "auto-gain-level", "value": "0"},
            {"name": "caller-controls", "value": "default"},
            {"name": "moderator-controls", "value": "moderator"},
            {"name": "muted-sound", "value": "conference/conf-muted.wav"},
            {"name": "unmuted-sound", "value": "conference/conf-unmuted.wav"},
            {"name": "alone-sound", "value": "conference/conf-alone.wav"},
            {"name": "moh-sound", "value": "local_stream://default"},
            {"name": "enter-sound", "value": "tone_stream://%(200,0,500,600,700)"},
            {"name": "exit-sound", "value": "tone_stream://%(500,0,300,200,100,50,25)"},
            {"name": "kicked-sound", "value": "conference/conf-kicked.wav"},
            {"name": "locked-sound", "value": "conference/conf-locked.wav"},
            {"name": "is-locked-sound", "value": "conference/conf-is-locked.wav"},
            {"name": "is-unlocked-sound", "value": "conference/conf-is-unlocked.wav"},
            {"name": "pin-sound", "value": "conference/conf-pin.wav"},
            {"name": "bad-pin-sound", "value": "conference/conf-bad-pin.wav"},
            {"name": "caller-id-name", "value": ""},
            {"name": "caller-id-number", "value": ""},
            {"name": "comfort-noise", "value": "true"},
        ],
    },
    {
        "name": "cdquality",
        "params": [
            {"name": "domain", "value": ""},
            {"name": "rate", "value": "48000"},
            {"name": "interval", "value": "10"},
            {"name": "energy-level", "value": "15"},
            {"name": "auto-gain-level", "value": "0"},
            {"name": "caller-controls", "value": "default"},
            {"name": "moderator-controls", "value": "moderator"},
            {"name": "muted-sound", "value": "conference/conf-muted.wav"},
            {"name": "unmuted-sound", "value": "conference/conf-unmuted.wav"},
            {"name": "alone-sound", "value": "conference/conf-alone.wav"},
            {"name": "moh-sound", "value": "local_stream://default"},
            {"name": "enter-sound", "value": "tone_stream://%(200,0,500,600,700)"},
            {"name": "exit-sound", "value": "tone_stream://%(500,0,300,200,100,50,25)"},
            {"name": "kicked-sound", "value": "conference/conf-kicked.wav"},
            {"name": "locked-sound", "value": "conference/conf-locked.wav"},
            {"name": "is-locked-sound", "value": "conference/conf-is-locked.wav"},
            {"name": "is-unlocked-sound", "value": "conference/conf-is-unlocked.wav"},
            {"name": "pin-sound", "value": "conference/conf-pin.wav"},
            {"name": "bad-pin-sound", "value": "conference/conf-bad-pin.wav"},
            {"name": "caller-id-name", "value": ""},
            {"name": "caller-id-number", "value": ""},
            {"name": "comfort-noise", "value": "true"},
        ],
    },
    {
        "name": "video-mcu-stereo",
        "params": [
            {"name": "domain", "value": "$${domain}"},
            {"name": "rate", "value": "48000"},
            {"name": "channels", "value": "2"},
            {"name": "interval", "value": "20"},
            {"name": "energy-level", "value": "200"},
            {"name": "muted-sound", "value": "conference/conf-muted.wav"},
            {"name": "unmuted-sound", "value": "conference/conf-unmuted.wav"},
            {"name": "alone-sound", "value": "conference/conf-alone.wav"},
            {"name": "moh-sound", "value": "$${hold_music}"},
            {"name": "enter-sound", "value": "tone_stream://%(200,0,500,600,700)"},
            {"name": "exit-sound", "value": "tone_stream://%(500,0,300,200,100,50,25)"},
            {"name": "kicked-sound", "value": "conference/conf-kicked.wav"},
            {"name": "locked-sound", "value": "conference/conf-locked.wav"},
            {"name": "is-locked-sound", "value": "conference/conf-is-locked.wav"},
            {"name": "is-unlocked-sound", "value": "conference/conf-is-unlocked.wav"},
            {"name": "pin-sound", "value": "conference/conf-pin.wav"},
            {"name": "bad-pin-sound", "value": "conference/conf-bad-pin.wav"},
            {"name": "caller-id-name", "value": "$${outbound_caller_name}"},
            {"name": "caller-id-number", "value": "$${outbound_caller_id}"},
            {"name": "comfort-noise", "value": "false"},
            {
                "name": "conference-flags",
                "value": "livearray-json-status|json-events|video-floor-only|rfc-4579|livearray-sync|minimize-video-encoding|manage-inbound-video-bitrate|video-required-for-canvas|video-mute-exit-canvas|mute-detect",
            },
            {"name": "video-auto-floor-msec", "value": "1000"},
            {"name": "video-mode", "value": "mux"},
            {"name": "video-layout-name", "value": "3x3"},
            {"name": "video-layout-name", "value": "group:grid"},
            {"name": "video-canvas-size", "value": "1920x1080"},
            {"name": "video-canvas-bgcolor", "value": "#333333"},
            {"name": "video-layout-bgcolor", "value": "#000000"},
            {"name": "video-codec-bandwidth", "value": "3mb"},
            {"name": "video-fps", "value": "30"},
        ],
    },
    {
        "name": "video-mcu-stereo-720",
        "params": [
            {"name": "domain", "value": "$${domain}"},
            {"name": "rate", "value": "48000"},
            {"name": "channels", "value": "2"},
            {"name": "interval", "value": "20"},
            {"name": "energy-level", "value": "200"},
            {"name": "muted-sound", "value": "conference/conf-muted.wav"},
            {"name": "unmuted-sound", "value": "conference/conf-unmuted.wav"},
            {"name": "alone-sound", "value": "conference/conf-alone.wav"},
            {"name": "moh-sound", "value": "$${hold_music}"},
            {"name": "enter-sound", "value": "tone_stream://%(200,0,500,600,700)"},
            {"name": "exit-sound", "value": "tone_stream://%(500,0,300,200,100,50,25)"},
            {"name": "kicked-sound", "value": "conference/conf-kicked.wav"},
            {"name": "locked-sound", "value": "conference/conf-locked.wav"},
            {"name": "is-locked-sound", "value": "conference/conf-is-locked.wav"},
            {"name": "is-unlocked-sound", "value": "conference/conf-is-unlocked.wav"},
            {"name": "pin-sound", "value": "conference/conf-pin.wav"},
            {"name": "bad-pin-sound", "value": "conference/conf-bad-pin.wav"},
            {"name": "caller-id-name", "value": "$${outbound_caller_name}"},
            {"name": "caller-id-number", "value": "$${outbound_caller_id}"},
            {"name": "comfort-noise", "value": "false"},
            {
                "name": "conference-flags",
                "value": "livearray-json-status|json-events|video-floor-only|rfc-4579|livearray-sync|minimize-video-encoding|manage-inbound-video-bitrate|video-required-for-canvas|video-mute-exit-canvas|mute-detect",
            },
            {"name": "video-auto-floor-msec", "value": "1000"},
            {"name": "video-mode", "value": "mux"},
            {"name": "video-layout-name", "value": "3x3"},
            {"name": "video-layout-name", "value": "group:grid"},
            {"name": "video-canvas-size", "value": "1280x720"},
            {"name": "video-canvas-bgcolor", "value": "#333333"},
            {"name": "video-layout-bgcolor", "value": "#000000"},
            {"name": "video-codec-bandwidth", "value": "3mb"},
            {"name": "video-fps", "value": "30"},
        ],
    },
    {
        "name": "video-mcu-stereo-480",
        "params": [
            {"name": "domain", "value": "$${domain}"},
            {"name": "rate", "value": "48000"},
            {"name": "channels", "value": "2"},
            {"name": "interval", "value": "20"},
            {"name": "energy-level", "value": "200"},
            {"name": "muted-sound", "value": "conference/conf-muted.wav"},
            {"name": "unmuted-sound", "value": "conference/conf-unmuted.wav"},
            {"name": "alone-sound", "value": "conference/conf-alone.wav"},
            {"name": "moh-sound", "value": "$${hold_music}"},
            {"name": "enter-sound", "value": "tone_stream://%(200,0,500,600,700)"},
            {"name": "exit-sound", "value": "tone_stream://%(500,0,300,200,100,50,25)"},
            {"name": "kicked-sound", "value": "conference/conf-kicked.wav"},
            {"name": "locked-sound", "value": "conference/conf-locked.wav"},
            {"name": "is-locked-sound", "value": "conference/conf-is-locked.wav"},
            {"name": "is-unlocked-sound", "value": "conference/conf-is-unlocked.wav"},
            {"name": "pin-sound", "value": "conference/conf-pin.wav"},
            {"name": "bad-pin-sound", "value": "conference/conf-bad-pin.wav"},
            {"name": "caller-id-name", "value": "$${outbound_caller_name}"},
            {"name": "caller-id-number", "value": "$${outbound_caller_id}"},
            {"name": "comfort-noise", "value": "false"},
            {
                "name": "conference-flags",
                "value": "livearray-json-status|json-events|video-floor-only|rfc-4579|livearray-sync|minimize-video-encoding|manage-inbound-video-bitrate|video-required-for-canvas|video-mute-exit-canvas|mute-detect",
            },
            {"name": "video-auto-floor-msec", "value": "1000"},
            {"name": "video-mode", "value": "mux"},
            {"name": "video-layout-name", "value": "3x3"},
            {"name": "video-layout-name", "value": "group:grid"},
            {"name": "video-canvas-size", "value": "640x480"},
            {"name": "video-canvas-bgcolor", "value": "#333333"},
            {"name": "video-layout-bgcolor", "value": "#000000"},
            {"name": "video-codec-bandwidth", "value": "3mb"},
            {"name": "video-fps", "value": "30"},
        ],
    },
    {
        "name": "video-mcu-stereo-320",
        "params": [
            {"name": "domain", "value": "$${domain}"},
            {"name": "rate", "value": "48000"},
            {"name": "channels", "value": "2"},
            {"name": "interval", "value": "20"},
            {"name": "energy-level", "value": "200"},
            {"name": "muted-sound", "value": "conference/conf-muted.wav"},
            {"name": "unmuted-sound", "value": "conference/conf-unmuted.wav"},
            {"name": "alone-sound", "value": "conference/conf-alone.wav"},
            {"name": "moh-sound", "value": "$${hold_music}"},
            {"name": "enter-sound", "value": "tone_stream://%(200,0,500,600,700)"},
            {"name": "exit-sound", "value": "tone_stream://%(500,0,300,200,100,50,25)"},
            {"name": "kicked-sound", "value": "conference/conf-kicked.wav"},
            {"name": "locked-sound", "value": "conference/conf-locked.wav"},
            {"name": "is-locked-sound", "value": "conference/conf-is-locked.wav"},
            {"name": "is-unlocked-sound", "value": "conference/conf-is-unlocked.wav"},
            {"name": "pin-sound", "value": "conference/conf-pin.wav"},
            {"name": "bad-pin-sound", "value": "conference/conf-bad-pin.wav"},
            {"name": "caller-id-name", "value": "$${outbound_caller_name}"},
            {"name": "caller-id-number", "value": "$${outbound_caller_id}"},
            {"name": "comfort-noise", "value": "false"},
            {
                "name": "conference-flags",
                "value": "livearray-json-status|json-events|video-floor-only|rfc-4579|livearray-sync|minimize-video-encoding|manage-inbound-video-bitrate|video-required-for-canvas|video-mute-exit-canvas|mute-detect",
            },
            {"name": "video-auto-floor-msec", "value": "1000"},
            {"name": "video-mode", "value": "mux"},
            {"name": "video-layout-name", "value": "3x3"},
            {"name": "video-layout-name", "value": "group:grid"},
            {"name": "video-canvas-size", "value": "480x320"},
            {"name": "video-canvas-bgcolor", "value": "#333333"},
            {"name": "video-layout-bgcolor", "value": "#000000"},
            {"name": "video-codec-bandwidth", "value": "3mb"},
            {"name": "video-fps", "value": "30"},
        ],
    },
    {
        "name": "sla",
        "params": [
            {"name": "domain", "value": ""},
            {"name": "rate", "value": "16000"},
            {"name": "interval", "value": "20"},
            {"name": "caller-controls", "value": "none"},
            {"name": "energy-level", "value": "200"},
            {"name": "moh-sound", "value": "silence"},
            {"name": "comfort-noise", "value": "true"},
        ],
    },
    {
        "name": "page",
        "params": [
            {"name": "domain", "value": ""},
            {"name": "rate", "value": "8000"},
            {"name": "interval", "value": "20"},
            {"name": "energy-level", "value": "300"},
            {"name": "caller-controls", "value": "page"},
            {"name": "moderator-controls", "value": "moderator"},
            {"name": "muted-sound", "value": "conference/conf-muted.wav"},
            {"name": "unmuted-sound", "value": "conference/conf-unmuted.wav"},
            {"name": "kicked-sound", "value": "conference/conf-kicked.wav"},
            {"name": "locked-sound", "value": "conference/conf-locked.wav"},
            {"name": "is-locked-sound", "value": "conference/conf-is-locked.wav"},
            {"name": "is-unlocked-sound", "value": "conference/conf-is-unlocked.wav"},
            {"name": "caller-id-name", "value": ""},
            {"name": "caller-id-number", "value": ""},
            {"name": "comfort-noise", "value": "true"},
            {"name": "terminate-on-silence", "value": "30"},
        ],
    },
]

MOH_ENTRIES: list[dict] = [
    {
        "name": "default",
        "path": "$${sounds_dir}/music/default/8000",
        "rate": 8000,
        "shuffle": "true",
        "channels": 1,
        "interval": 20,
        "timer": "soft",
        "chime_list": None,
        "chime_freq": None,
        "chime_max": None,
    },
    {
        "name": "default",
        "path": "$${sounds_dir}/music/default/16000",
        "rate": 16000,
        "shuffle": "true",
        "channels": 1,
        "interval": 20,
        "timer": "soft",
        "chime_list": None,
        "chime_freq": None,
        "chime_max": None,
    },
    {
        "name": "default",
        "path": "$${sounds_dir}/music/default/32000",
        "rate": 32000,
        "shuffle": "true",
        "channels": 1,
        "interval": 20,
        "timer": "soft",
        "chime_list": None,
        "chime_freq": None,
        "chime_max": None,
    },
    {
        "name": "default",
        "path": "$${sounds_dir}/music/default/48000",
        "rate": 48000,
        "shuffle": "true",
        "channels": 1,
        "interval": 10,
        "timer": "soft",
        "chime_list": None,
        "chime_freq": None,
        "chime_max": None,
    },
]


async def seed_access_controls(session, domain_id) -> int:
    from sqlalchemy import select as _select

    from app.access_controls.models import AccessControl as _A
    from app.access_controls.models import AccessControlNode as _N

    n = 0
    for a in ACCESS_CONTROLS:
        r = await session.execute(
            _select(_A).where(_A.domain_id == domain_id, _A.access_control_name == a["name"])
        )
        obj = r.scalar_one_or_none()
        if obj is None:
            obj = _A(
                domain_id=domain_id,
                access_control_name=a["name"],
                access_control_default=a["default"],
                access_control_description=a.get("description"),
            )
            session.add(obj)
            await session.flush()
            await session.refresh(obj)
            n += 1
        for nd in a.get("nodes", []):
            rr = await session.execute(
                _select(_N).where(
                    _N.access_control_uuid == obj.access_control_uuid, _N.node_cidr == nd["cidr"]
                )
            )
            if rr.scalar_one_or_none():
                continue
            session.add(
                _N(
                    access_control_uuid=obj.access_control_uuid,
                    domain_id=domain_id,
                    node_type=nd["type"],
                    node_cidr=nd["cidr"],
                    node_description=nd.get("description"),
                )
            )
            n += 1
    await session.flush()
    return n


async def seed_conference_profiles(session, domain_id) -> int:
    from sqlalchemy import select as _select

    from app.conference_profiles.models import ConferenceProfile as _C
    from app.conference_profiles.models import ConferenceProfileParam as _P

    n = 0
    for c in CONFERENCE_PROFILES:
        r = await session.execute(
            _select(_C).where(_C.domain_id == domain_id, _C.profile_name == c["name"])
        )
        obj = r.scalar_one_or_none()
        if obj is None:
            obj = _C(domain_id=domain_id, profile_name=c["name"], profile_enabled=True)
            session.add(obj)
            await session.flush()
            await session.refresh(obj)
            n += 1
        for p in c.get("params", []):
            if not p.get("name"):
                continue
            rr = await session.execute(
                _select(_P).where(
                    _P.conference_profile_uuid == obj.conference_profile_uuid,
                    _P.profile_param_name == p["name"],
                )
            )
            if rr.scalar_one_or_none():
                continue
            session.add(
                _P(
                    conference_profile_uuid=obj.conference_profile_uuid,
                    domain_id=domain_id,
                    profile_param_name=p["name"],
                    profile_param_value=p.get("value") or "",
                    profile_param_enabled=True,
                )
            )
            n += 1
    await session.flush()
    return n


async def seed_music_on_hold(session, domain_id) -> int:
    from sqlalchemy import select as _select

    from app.music_on_hold.models import MusicOnHold as _M

    n = 0
    for m in MOH_ENTRIES:
        label = f"{m['name']}_{m['rate']}"
        r = await session.execute(
            _select(_M).where(
                _M.domain_id == domain_id, _M.music_name == m["name"], _M.music_rate == m["rate"]
            )
        )
        if r.scalar_one_or_none():
            continue
        session.add(
            _M(
                domain_id=domain_id,
                music_name=m["name"],
                music_path=m["path"],
                music_rate=m["rate"],
                music_shuffle=m.get("shuffle") or "false",
                music_channels=m.get("channels") or 1,
                music_interval=m.get("interval") or 20,
                music_timer_name=m.get("timer"),
                music_chime_list=m.get("chime_list"),
                music_chime_freq=m.get("chime_freq"),
                music_chime_max=m.get("chime_max"),
                enabled=True,
                description=f"MOH {label} (seeded)",
            )
        )
        n += 1
    await session.flush()
    return n
