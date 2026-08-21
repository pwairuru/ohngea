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
        user = etree.SubElement(users_el, "user", id=ext.extension)
        p = etree.SubElement(user, "params")
        etree.SubElement(p, "param", name="password", value="1234")
        etree.SubElement(p, "param", name="vm-password", value=ext.extension)
        if hasattr(ext, "effective_caller_id_name") and ext.effective_caller_id_name:
            etree.SubElement(p, "param", name="effective_caller_id_name", value=ext.effective_caller_id_name)
        if hasattr(ext, "effective_caller_id_number") and ext.effective_caller_id_number:
            etree.SubElement(p, "param", name="effective_caller_id_number", value=ext.effective_caller_id_number)
        vars_el = etree.SubElement(user, "variables")
        etree.SubElement(vars_el, "variable", name="toll_allow", value="domestic,international,local")
        etree.SubElement(vars_el, "variable", name="accountcode", value=str(ext.domain_id))
        etree.SubElement(vars_el, "variable", name="user_context", value=domain_name)
        etree.SubElement(vars_el, "variable", name="effective_caller_id_name", value=getattr(ext, "effective_caller_id_name", ""))
    # also add direct users under domain for mod_xml_curl compatibility
    for ext in extensions:
        user = etree.SubElement(domain, "user", id=ext.extension)
        p = etree.SubElement(user, "params")
        etree.SubElement(p, "param", name="password", value="1234")
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


def build_empty_configuration_xml() -> bytes:
    root = etree.Element("document", type="freeswitch/xml")
    section = etree.SubElement(root, "section", name="configuration")
    return etree.tostring(root, xml_declaration=True, encoding="UTF-8")
