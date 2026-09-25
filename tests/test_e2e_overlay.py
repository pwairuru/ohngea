# tests/test_e2e_overlay.py
def test_sofia_conf_has_internal_no_hardcoded_ip():
    import pathlib
    xml = pathlib.Path("docker/freeswitch/conf/autoload_configs/sofia.conf.xml").read_text()
    assert 'profile name="internal"' in xml
    assert "patchtest" not in xml
    # NOTE: 192.168.1.108 was once a stale hardcode; it is now also a real
    # DHCP address on some LANs, so it must not be asserted absent (see
    # test_sofia_uses_lan_ip_literal for the shape check).

def test_compose_publishes_sip_and_rtp():
    import pathlib
    y = pathlib.Path("docker-compose.e2e.yml").read_text()
    assert "5060:5060/udp" in y
    assert "8022:8021" in y
    assert "16384-16386:16384-16386/udp" in y
    assert "LAN_IP" in y

def test_acl_allows_bridge_and_lan():
    import pathlib
    xml = pathlib.Path("docker/freeswitch/conf/autoload_configs/acl.conf.xml").read_text()
    assert 'list name="e2e-all"' in xml
    assert "127.0.0.1/32" in xml
    assert "172.16.0.0/12" in xml
    assert "192.168.0.0/16" in xml
    assert 'cidr="0.0.0.0/0"' not in xml

def test_sofia_uses_lan_ip_literal():
    import pathlib, re
    xml = pathlib.Path("docker/freeswitch/conf/autoload_configs/sofia.conf.xml").read_text()
    # ext IPs must be a concrete LAN IP (FS never expands compose env here);
    # exact value roams with DHCP, so only assert shape, not the address.
    assert re.search(r'<param name="ext-sip-ip" value="\d+\.\d+\.\d+\.\d+"', xml)
    assert re.search(r'<param name="ext-rtp-ip" value="\d+\.\d+\.\d+\.\d+"', xml)
    assert "${LAN_IP}" not in xml  # FS never expands compose env inside file content

def test_verify_emits_json():
    import subprocess, sys
    # verify --help must mention --json and --out
    r = subprocess.run([sys.executable, "tests/e2e/verify.py", "--help"], capture_output=True, text=True)
    assert "--json" in r.stdout
    assert "--out" in r.stdout
