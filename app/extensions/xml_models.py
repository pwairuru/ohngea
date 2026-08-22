"""pydantic-xml models for FreeSWITCH directory XML."""

from typing import Optional

from pydantic_xml import BaseXmlModel, attr, element


class ParamsXml(BaseXmlModel, tag="params"):
    # key-value params inside user
    pass


class ParamXml(BaseXmlModel, tag="param"):
    name: str = attr()
    value: str = attr()


class UserXml(BaseXmlModel, tag="user"):
    id: str = attr()
    params: Optional[list[ParamXml]] = element(tag="param", default=None)

    @classmethod
    def from_db(cls, db) -> "UserXml":
        # Map Extension fields to FS directory user with full parity
        pwd = getattr(db, "password", None) or "1234"
        params = [
            ParamXml(name="password", value=str(pwd)),
            ParamXml(name="vm-password", value=str(pwd)),
            ParamXml(name="reverse-auth-user", value=str(db.extension)),
            ParamXml(name="reverse-auth-pass", value=str(pwd)),
        ]
        if getattr(db, "effective_caller_id_name", None):
            params.append(ParamXml(name="effective_caller_id_name", value=db.effective_caller_id_name))
        if getattr(db, "effective_caller_id_number", None):
            params.append(ParamXml(name="effective_caller_id_number", value=db.effective_caller_id_number))
        if getattr(db, "outbound_caller_id_name", None):
            params.append(ParamXml(name="outbound_caller_id_name", value=db.outbound_caller_id_name))
        if getattr(db, "outbound_caller_id_number", None):
            params.append(ParamXml(name="outbound_caller_id_number", value=db.outbound_caller_id_number))
        if getattr(db, "mwi_account", None):
            params.append(ParamXml(name="MWI-Account", value=db.mwi_account))
        if getattr(db, "auth_acl", None):
            params.append(ParamXml(name="auth-acl", value=db.auth_acl))
        dial_string = getattr(db, "dial_string", None)
        if dial_string:
            params.append(ParamXml(name="dial-string", value=dial_string))
        return cls(id=db.extension, params=params)


class DomainXml(BaseXmlModel, tag="domain"):
    name: str = attr()
    params: Optional[list[ParamXml]] = element(tag="param", default=None)
    users: list[UserXml] = element(tag="user", default_factory=list)


class SectionXml(BaseXmlModel, tag="section"):
    name: str = attr()
    domains: list[DomainXml] = element(tag="domain", default_factory=list)


class DocumentXml(BaseXmlModel, tag="document"):
    type: str = attr(default="freeswitch/xml")
    sections: list[SectionXml] = element(tag="section", default_factory=list)
