from typing import Optional
from pydantic_xml import BaseXmlModel, attr, element


class ActionXml(BaseXmlModel, tag="action"):
    application: str = attr()
    data: str = attr(default="")


class ConditionXml(BaseXmlModel, tag="condition"):
    field: str = attr(default="destination_number")
    expression: str = attr()
    actions: list[ActionXml] = element(tag="action", default_factory=list)


class ExtensionXml(BaseXmlModel, tag="extension"):
    name: str = attr()
    continue_: bool = attr(default=False)
    conditions: list[ConditionXml] = element(tag="condition", default_factory=list)
