from lxml import etree
from typing import List
from uuid import UUID


def build_directory_xml(domain_name: str, extensions) -> bytes:
    root = etree.Element("document", type="freeswitch/xml")
    section = etree.SubElement(root, "section", name="directory")
    domain = etree.SubElement(section, "domain", name=domain_name)
    params = etree.SubElement(domain, "params")
    etree.SubElement(params, "param", name="dial-string", value="{presence_id=${dialed_user}@${dialed_domain}}${sofia_contact(*/${dialed_user}@${dialed_domain})}")
    # groups
    groups = etree.SubElement(domain, "groups")
    group = etree.SubElement(groups, "group", name="default")
    users_el = etree.SubElement(group, "users")
    for ext in extensions:
        # attributes on user element
        user_attrs = {"id": str(ext.extension)}
        cidr = getattr(ext, "cidr", None)
        if cidr:
            user_attrs["cidr"] = cidr
        number_alias = getattr(ext, "number_alias", None)
        if number_alias:
            user_attrs["number-alias"] = str(number_alias)
        user = etree.SubElement(users_el, "user", **user_attrs)
        p = etree.SubElement(user, "params")
        # password — real SIP password, fallback 1234 only if None (legacy compat)
        pwd = getattr(ext, "password", None) or "1234"
        etree.SubElement(p, "param", name="password", value=str(pwd))
        # reverse auth (legacy does)
        etree.SubElement(p, "param", name="reverse-auth-user", value=str(ext.extension))
        etree.SubElement(p, "param", name="reverse-auth-pass", value=str(pwd))
        # vm
        vm_pwd = getattr(ext, "password", None) or str(ext.extension)
        # if voicemail password separate we could use that, but fallback
        etree.SubElement(p, "param", name="vm-password", value=str(vm_pwd))
        # vm-enabled default true
        # Could check voicemail_enabled via separate lookup, but use extension.enabled
        vm_enabled = "true" if getattr(ext, "enabled", True) else "false"
        etree.SubElement(p, "param", name="vm-enabled", value=vm_enabled)
        # voicemail email handling if present
        vm_mailto = getattr(ext, "voicemail_mail_to", None) or getattr(ext, "vm_mailto", None)
        if vm_mailto:
            etree.SubElement(p, "param", name="vm-email-all-messages", value="true")
            vf = getattr(ext, "voicemail_file", None) or getattr(ext, "voicemail_file", None) or "true"
            # legacy: attach if voicemail_file == attach
            attach = "true" if str(vf) in ("attach","true") else "false"
            etree.SubElement(p, "param", name="vm-attach-file", value=attach)
            keep = getattr(ext, "voicemail_local_after_email", None)
            if keep is not None:
                val = "true" if str(keep).lower() in ("true","1") else "false"
            else:
                val = "true"
            etree.SubElement(p, "param", name="vm-keep-local-after-email", value=val)
            etree.SubElement(p, "param", name="vm-mailto", value=str(vm_mailto))
        mwi = getattr(ext, "mwi_account", None)
        if mwi:
            etree.SubElement(p, "param", name="MWI-Account", value=str(mwi))
        auth_acl = getattr(ext, "auth_acl", None)
        if auth_acl:
            etree.SubElement(p, "param", name="auth-acl", value=str(auth_acl))
        dir_ext_visible = getattr(ext, "directory_exten_visible", None)
        if dir_ext_visible is not None:
            etree.SubElement(p, "param", name="directory-exten-visible", value=str(dir_ext_visible).lower())
        # dial-string
        dial_string = getattr(ext, "dial_string", None)
        if dial_string:
            etree.SubElement(p, "param", name="dial-string", value=str(dial_string))
        else:
            # default with call_timeout
            ct = getattr(ext, "call_timeout", 30) or 30
            ds = f"{{sip_invite_domain=${{domain_name}},leg_timeout={ct},presence_id=${{dialed_user}}@${{dialed_domain}}}}${{sofia_contact(${{dialed_user}}@${{dialed_domain}})}}"
            etree.SubElement(p, "param", name="dial-string", value=ds)

        vars_el = etree.SubElement(user, "variables")
        etree.SubElement(vars_el, "variable", name="domain_name", value=str(domain_name))
        etree.SubElement(vars_el, "variable", name="domain_uuid", value=str(getattr(ext, "domain_id", "")))
        etree.SubElement(vars_el, "variable", name="extension_uuid", value=str(getattr(ext, "extension_uuid", "")))
        # helper to add variable if present
        def add_var(name, val):
            if val is not None and str(val) != "":
                etree.SubElement(vars_el, "variable", name=name, value=str(val))
        add_var("call_group", getattr(ext, "call_group", None))
        add_var("user_record", getattr(ext, "user_record", None))
        add_var("hold_music", getattr(ext, "hold_music", None))
        # toll_allow defaults to allow all? legacy uses toll_allow from DB
        ta = getattr(ext, "toll_allow", None)
        if ta:
            add_var("toll_allow", ta)
        else:
            etree.SubElement(vars_el, "variable", name="toll_allow", value="domestic,international,local")
        add_var("call_timeout", getattr(ext, "call_timeout", None))
        # accountcode
        ac = getattr(ext, "accountcode", None)
        if ac:
            add_var("accountcode", ac)
        else:
            add_var("accountcode", str(getattr(ext, "domain_id", "")))
        add_var("user_context", getattr(ext, "user_context", None) or domain_name)
        add_var("effective_caller_id_name", getattr(ext, "effective_caller_id_name", None))
        add_var("effective_caller_id_number", getattr(ext, "effective_caller_id_number", None))
        add_var("outbound_caller_id_name", getattr(ext, "outbound_caller_id_name", None))
        add_var("outbound_caller_id_number", getattr(ext, "outbound_caller_id_number", None))
        add_var("emergency_caller_id_name", getattr(ext, "emergency_caller_id_name", None))
        add_var("emergency_caller_id_number", getattr(ext, "emergency_caller_id_number", None))
        # directory full name
        dfn = getattr(ext, "directory_first_name", None)
        dln = getattr(ext, "directory_last_name", None)
        if dfn or dln:
            full = f"{dfn or ''} {dln or ''}".strip()
            add_var("directory_full_name", full)
        # directory visible
        dv = getattr(ext, "directory_visible", None)
        if dv is not None:
            etree.SubElement(vars_el, "variable", name="directory-visible", value=str(dv).lower())
        # limit
        add_var("limit_max", getattr(ext, "limit_max", None) or "5")
        add_var("limit_destination", getattr(ext, "limit_destination", None) or "!USER_BUSY")
        # sip
        add_var("sip-force-contact", getattr(ext, "sip_force_contact", None))
        add_var("sip-force-expires", getattr(ext, "sip_force_expires", None))
        add_var("nibble_account", getattr(ext, "nibble_account", None))
        # bypass media mapping
        sbm = getattr(ext, "sip_bypass_media", None)
        if sbm == "bypass-media":
            etree.SubElement(vars_el, "variable", name="bypass_media", value="true")
        elif sbm == "bypass-media-after-bridge":
            etree.SubElement(vars_el, "variable", name="bypass_media_after_bridge", value="true")
        elif sbm == "proxy-media":
            etree.SubElement(vars_el, "variable", name="proxy_media", value="true")
        add_var("absolute_codec_string", getattr(ext, "absolute_codec_string", None))
        # forward
        for fwd in ["forward_all","forward_busy","forward_no_answer","forward_user_not_registered"]:
            enabled = getattr(ext, f"{fwd}_enabled", None)
            dest = getattr(ext, f"{fwd}_destination", None)
            if enabled:
                etree.SubElement(vars_el, "variable", name=f"{fwd}_enabled", value=str(enabled).lower())
            if dest:
                etree.SubElement(vars_el, "variable", name=f"{fwd}_destination", value=str(dest))
        if getattr(ext, "do_not_disturb", None):
            etree.SubElement(vars_el, "variable", name="do_not_disturb", value=str(getattr(ext,"do_not_disturb")).lower())
        # call groups already, but also file extension_type lang etc as vars for dialplan use
        add_var("extension_language", getattr(ext, "extension_language", None))
        add_var("extension_dialect", getattr(ext, "extension_dialect", None))
        add_var("extension_voice", getattr(ext, "extension_voice", None))
        add_var("extension_type", getattr(ext, "extension_type", None))
        # also ensure effective caller id duplicated as param+var
    return etree.tostring(root, xml_declaration=True, encoding="UTF-8", pretty_print=True)


def build_dialplan_xml(context: str, dialplans) -> bytes:
    root = etree.Element("document", type="freeswitch/xml")
    section = etree.SubElement(root, "section", name="dialplan", description="PBX Dialplan")
    ctx = etree.SubElement(section, "context", name=context)
    for dp in dialplans:
        ext = etree.SubElement(ctx, "extension", name=dp.name, **({"continue": "true"} if False else {}))
        # fetch details via attribute? Expect dp.details list if attached, else generic
        details = getattr(dp, "details", []) or getattr(dp, "_details", [])
        # if we only have Dialplan without details joined, try to use empty
        if not details and hasattr(dp, "dialplans_details"):
            details = dp.dialplans_details  # placeholder
        # fallback: create condition/action from DialplanDetail rows will be passed as tuples
        # We'll attempt to group by order: condition vs action
        current_condition = None
        for det in details:
            tag = getattr(det, "tag", det.get("tag") if isinstance(det, dict) else "condition")
            typ = getattr(det, "type", det.get("type") if isinstance(det, dict) else "")
            data = getattr(det, "data", det.get("data") if isinstance(det, dict) else "")
            if tag == "condition":
                # typ is field like destination_number, data is expression
                current_condition = etree.SubElement(ext, "condition", field=typ or "destination_number", expression=data or ".*")
            elif tag in ("action", "anti-action"):
                target = current_condition if current_condition is not None else ext
                etree.SubElement(target, "action", application=typ or "transfer", data=data or "")
            elif tag == "param":
                etree.SubElement(ext, "param", name=typ, value=data)
        # if no details, add default transfer to extension
        if not details:
            cond = etree.SubElement(ext, "condition", field="destination_number", expression=f"^{dp.name}$")
            etree.SubElement(cond, "action", application="transfer", data=f"{dp.name} XML {context}")
    return etree.tostring(root, xml_declaration=True, encoding="UTF-8", pretty_print=True)


def build_not_found_xml() -> bytes:
    root = etree.Element("document", type="freeswitch/xml")
    section = etree.SubElement(root, "section", name="result")
    etree.SubElement(section, "result", status="not found")
    return etree.tostring(root, xml_declaration=True, encoding="UTF-8")


def build_acl_xml(acls) -> bytes:
    """Build acl.conf XML from AccessControl rows with nodes.

    Each ACL becomes <list name= default=> containing <node> entries.
    nodes expected as list of AccessControlNode or dicts with node_type/node_cidr.
    acls is iterable of (acl, nodes) tuples or acl objects with .nodes.
    """
    root = etree.Element("document", type="freeswitch/xml")
    section = etree.SubElement(root, "section", name="configuration")
    conf = etree.SubElement(section, "configuration", name="acl.conf", description="Access Controls")
    network_lists = etree.SubElement(conf, "network-lists")
    for item in acls:
        # support (acl, nodes) tuple
        if isinstance(item, tuple) and len(item) == 2:
            acl, nodes = item
        else:
            acl = item
            nodes = getattr(acl, "nodes", []) or getattr(acl, "_nodes", []) or []
            # also try attribute nodes
            if not nodes and hasattr(acl, "access_control_nodes"):
                nodes = acl.access_control_nodes
        name = getattr(acl, "access_control_name", None) or getattr(acl, "name", "unknown")
        default = getattr(acl, "access_control_default", None) or getattr(acl, "default", "deny")
        lst = etree.SubElement(network_lists, "list", name=str(name), default=str(default))
        for n in nodes:
            t = getattr(n, "node_type", None) or (n.get("node_type") if isinstance(n, dict) else None) or "allow"
            cidr = getattr(n, "node_cidr", None) or (n.get("node_cidr") if isinstance(n, dict) else None) or "0.0.0.0/0"
            etree.SubElement(lst, "node", type=str(t), cidr=str(cidr))
    return etree.tostring(root, xml_declaration=True, encoding="UTF-8", pretty_print=True)


def build_gateway_xml(gateways, domain_name: str = "") -> bytes:
    """Build sofia.conf gateway XML for mod_xml_curl.

    FS expects document type freeswitch/xml -> section configuration
    -> configuration name=sofia.conf -> gateways -> gateway name=.
    All 30 legacy fields are rendered when present, matching
    legacy PBX resources/switch.php save_gateway_xml() plus extended
    ping/channel/codec fields.
    """
    root = etree.Element("document", type="freeswitch/xml")
    section = etree.SubElement(root, "section", name="configuration")
    conf = etree.SubElement(section, "configuration", name="sofia.conf", description="Sofia Gateways")
    gateways_el = etree.SubElement(conf, "gateways")
    for gw in gateways:
        name = getattr(gw, "gateway", None) or getattr(gw, "name", "gw")
        g_el = etree.SubElement(gateways_el, "gateway", name=str(name))

        def _bool_str(v):
            if isinstance(v, bool):
                return "true" if v else "false"
            if v is None:
                return None
            s = str(v).lower()
            if s in ("true", "false", "1", "0", "t", "f"):
                return "true" if s in ("true", "1", "t") else "false"
            return str(v)

        # helper to add param if value present
        def _add(param, value, is_bool=False):
            if value is None or value == "":
                return
            if is_bool:
                bv = _bool_str(value)
                if bv is None:
                    return
                etree.SubElement(g_el, "param", name=param, value=bv)
            else:
                etree.SubElement(g_el, "param", name=param, value=str(value))

        # core auth
        _add("username", getattr(gw, "username", None))
        _add("password", getattr(gw, "password", None))
        _add("auth-username", getattr(gw, "auth_username", None))
        _add("realm", getattr(gw, "realm", None))
        _add("distinct-to", getattr(gw, "distinct_to", None), is_bool=True)
        _add("from-user", getattr(gw, "from_user", None))
        _add("from-domain", getattr(gw, "from_domain", None))
        # proxy / transport
        _add("proxy", getattr(gw, "proxy", None))
        _add("register-proxy", getattr(gw, "register_proxy", None))
        _add("outbound-proxy", getattr(gw, "outbound_proxy", None))
        _add("register-transport", getattr(gw, "register_transport", None))
        _add("contact-params", getattr(gw, "contact_params", None))
        _add("extension", getattr(gw, "extension", None))
        _add("extension-in-contact", getattr(gw, "extension_in_contact", None))
        # timers
        _add("expire-seconds", getattr(gw, "expire_seconds", None))
        _add("retry-seconds", getattr(gw, "retry_seconds", None))
        _add("ping", getattr(gw, "ping", None))
        _add("ping-min", getattr(gw, "ping_min", None))
        _add("ping-max", getattr(gw, "ping_max", None))
        # booleans
        _add("contact-in-ping", getattr(gw, "contact_in_ping", None), is_bool=True)
        _add("caller-id-in-from", getattr(gw, "caller_id_in_from", None), is_bool=True)
        _add("supress-cng", getattr(gw, "supress_cng", None), is_bool=True)
        # sip / codec
        _add("sip-cid-type", getattr(gw, "sip_cid_type", None))
        # codec-prefs legacy is space/comma list
        _add("codec-prefs", getattr(gw, "codec_prefs", None))
        _add("channels", getattr(gw, "channels", None))
        # context/profile
        _add("context", getattr(gw, "context", None))
        # profile is not a gateway param but useful for debugging; FS uses directory profile separation externally
        # keep hostname as comment param if present (not standard FS param but helpful)
        _add("hostname", getattr(gw, "hostname", None))

        # register / enabled always
        # gateway_register is SA column "register"; property .register proxies it
        reg = getattr(gw, "gateway_register", None)
        if reg is None:
            reg = getattr(gw, "register", True)
        _add("register", reg, is_bool=True)
        # Note: enabled is DB gating; disabled gateways still rendered but register false; keep param for visibility
        _add("enabled", getattr(gw, "enabled", True), is_bool=True)
        if domain_name:
            _add("domain", domain_name)
    return etree.tostring(root, xml_declaration=True, encoding="UTF-8", pretty_print=True)


def build_vars_xml(vars_list) -> bytes:
    root = etree.Element("document", type="freeswitch/xml")
    section = etree.SubElement(root, "section", name="configuration")
    conf = etree.SubElement(section, "configuration", name="vars.xml", description="Vars")
    for v in vars_list:
        name = getattr(v, "var_name", None) or getattr(v, "name", "var")
        val = getattr(v, "var_value", None) or getattr(v, "value", "")
        etree.SubElement(conf, "X-PRE-PROCESS", cmd="set", data=f"{name}={val}")
    return etree.tostring(root, xml_declaration=True, encoding="UTF-8", pretty_print=True)


def build_empty_configuration_xml() -> bytes:
    root = etree.Element("document", type="freeswitch/xml")
    section = etree.SubElement(root, "section", name="configuration")
    return etree.tostring(root, xml_declaration=True, encoding="UTF-8")
