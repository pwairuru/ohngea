import pytest
from pydantic import ValidationError
from app.extensions.models import ExtensionBase
from uuid import uuid4


def test_extension_valid():
    e = ExtensionBase(extension="1001", effective_caller_id_name="Alice", domain_id=uuid4())
    assert e.extension == "1001"


def test_extension_invalid_short():
    with pytest.raises(ValidationError):
        ExtensionBase(extension="1", effective_caller_id_name="Alice", domain_id=uuid4())


def test_extension_invalid_chars():
    with pytest.raises(ValidationError):
        ExtensionBase(extension="abc", effective_caller_id_name="Alice", domain_id=uuid4())


def test_extension_enabled_default():
    e = ExtensionBase(extension="1001", effective_caller_id_name="Bob", domain_id=uuid4())
    assert e.enabled is True
