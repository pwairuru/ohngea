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
        # Map Extension fields to FS directory user
        params = [
            ParamXml(name="password", value="1234"),
            ParamXml(name="vm-password", value=getattr(db, "extension", "1234")),
        ]
        if db.effective_caller_id_name:
            params.append(ParamXml(name="effective_caller_id_name", value=db.effective_caller_id_name))
        if db.effective_caller_id_number:
            params.append(ParamXml(name="effective_caller_id_number", value=db.effective_caller_id_number))
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
