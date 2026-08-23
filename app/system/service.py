"""
Cross-platform system metrics for Linux, FreeBSD, OpenBSD, Darwin, Windows.

Mirrors legacy PHP classes:
  app/system/resources/classes/system_information.php (abstract)
  app/system/resources/classes/linux_system_information.php
  app/system/resources/classes/bsd_system_information.php

Supports psutil if installed (BSD-licensed, allowed) else pure stdlib+sysctl/procfs.
"""
from __future__ import annotations

import asyncio
import os
import platform
import re
import shutil
import subprocess
import sys
import time
from abc import ABC, abstractmethod
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

# Optional psutil fast path
try:
    import psutil  # type: ignore
    _HAS_PSUTIL = True
except Exception:
    _HAS_PSUTIL = False
    psutil = None  # type: ignore

# Module-level cache for CPU deltas (similar to PHP static $last)
_cpu_last: dict[str, Any] = {}
_cpu_last_per_core: dict[int, dict[str, int]] = {}
_last_global: dict[str, Any] = {}


def _run(cmd: str, timeout: float = 2.0) -> str:
    try:
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.DEVNULL, timeout=timeout).decode(errors="ignore").strip()
    except Exception:
        return ""


def _run_to_thread(cmd: str) -> str:
    return _run(cmd)


def _parse_human_bytes(b: int) -> str:
    for unit in ["B", "K", "M", "G", "T"]:
        if abs(b) < 1024:
            return f"{b:.1f}{unit}"
        b /= 1024  # type: ignore
    return f"{b:.1f}P"


def _uptime_human(seconds: float) -> str:
    mins, sec = divmod(int(seconds), 60)
    hrs, mins = divmod(mins, 60)
    days, hrs = divmod(hrs, 24)
    parts = []
    if days:
        parts.append(f"{days} days")
    if hrs:
        parts.append(f"{hrs} hours")
    if mins and not days:
        parts.append(f"{mins} mins")
    if not parts:
        parts.append(f"{sec} secs")
    return ", ".join(parts)


class BaseSystemInfo(ABC):
    """Abstract base, mirrors PHP system_information."""

    def get_load_average(self) -> list[float] | None:
        try:
            if hasattr(os, "getloadavg"):
                a, b, c = os.getloadavg()
                return [round(a, 2), round(b, 2), round(c, 2)]
        except Exception:
            pass
        return None

    def get_hostname(self) -> str:
        try:
            return platform.node() or os.uname().nodename  # type: ignore
        except Exception:
            return platform.node()

    def get_date(self) -> str:
        return datetime.now(timezone.utc).astimezone().strftime("%c %Z")

    def get_os_info(self) -> dict[str, str]:
        sysname = platform.system()
        release = platform.release()
        version = platform.version()
        # Try pretty OS name
        pretty = sysname
        try:
            if sysname == "Linux":
                # /etc/os-release
                p = Path("/etc/os-release")
                if p.exists():
                    txt = p.read_text(errors="ignore")
                    m = re.search(r'^PRETTY_NAME="?(.*?)"?$', txt, re.MULTILINE)
                    if m:
                        pretty = m.group(1)
                    else:
                        m2 = re.search(r'^NAME="?(.*?)"?$', txt, re.MULTILINE)
                        if m2:
                            pretty = m2.group(1)
                else:
                    # lsb_release fallback
                    out = _run("lsb_release -ds 2>/dev/null")
                    if out:
                        pretty = out.strip('"')
        except Exception:
            pass
        kernel = ""
        try:
            kernel = _run("uname -a 2>/dev/null") or f"{sysname} {release} {version}"
        except Exception:
            kernel = f"{sysname} {release}"
        return {"name": pretty, "version": release, "kernel": kernel, "type": sysname}

    def get_uptime(self) -> str:
        # Generic fallback: use `uptime` command
        out = _run("uptime 2>/dev/null")
        return out or "unknown"

    def get_uptime_seconds(self) -> float | None:
        out = self.get_uptime()
        # Try to parse from uptime output or OS-specific
        return None

    @abstractmethod
    def get_cpu_cores(self) -> int:
        ...

    @abstractmethod
    def get_cpu_percent(self) -> float:
        ...

    @abstractmethod
    def get_cpu_percent_per_core(self) -> list[float]:
        ...

    @abstractmethod
    def get_memory(self) -> dict[str, Any]:
        ...

    @abstractmethod
    def get_disk(self) -> list[dict[str, Any]]:
        ...

    def get_disk_summary(self) -> dict[str, Any]:
        disks = self.get_disk()
        total = sum(d.get("size_bytes", 0) for d in disks)
        used = sum(d.get("used_bytes", 0) for d in disks)
        avail = sum(d.get("avail_bytes", 0) for d in disks)
        return {"total_bytes": total, "used_bytes": used, "avail_bytes": avail, "disks": disks}

    def as_dict(self) -> dict[str, Any]:
        # Full snapshot for /system/info
        cores = self.get_cpu_cores()
        try:
            cpu_total = self.get_cpu_percent()
        except Exception:
            cpu_total = 0.0
        try:
            per_core = self.get_cpu_percent_per_core()
        except Exception:
            per_core = []
        mem = self.get_memory()
        disks = self.get_disk()
        load = self.get_load_average()
        uptime = self.get_uptime()
        osinfo = self.get_os_info()
        hostname = self.get_hostname()
        return {
            "os": osinfo,
            "hostname": hostname,
            "date": self.get_date(),
            "uptime": uptime,
            "load_avg": load,
            "cpu": {"cores": cores, "total_percent": cpu_total, "per_core": per_core},
            "memory": mem,
            "disk": disks,
            "python_version": platform.python_version(),
            "platform": sys.platform,
        }

    def metrics(self) -> dict[str, Any]:
        # Lightweight for polling
        try:
            cpu = self.get_cpu_percent()
        except Exception:
            cpu = 0.0
        try:
            per_core = self.get_cpu_percent_per_core()
        except Exception:
            per_core = []
        mem = self.get_memory()
        load = self.get_load_average()
        disks = self.get_disk()
        # summarize disk usage % average
        disk_percent = 0.0
        if disks:
            # take root or max
            for d in disks:
                if d.get("mount") in ("/", "/usr", "/var"):
                    disk_percent = d.get("use_percent", 0)
                    break
            else:
                disk_percent = disks[0].get("use_percent", 0)
        return {
            "cpu_total": cpu,
            "cpu_per_core": per_core,
            "cpu_cores": self.get_cpu_cores(),
            "load_avg": load,
            "memory": {"total": mem.get("total_bytes"), "used": mem.get("used_bytes"), "available": mem.get("available_bytes"), "percent": mem.get("percent"), "free": mem.get("free_bytes")},
            "disk_percent": disk_percent,
            "disks": disks,
            "uptime": self.get_uptime(),
            "hostname": self.get_hostname(),
        }


class LinuxSystemInfo(BaseSystemInfo):
    def get_cpu_cores(self) -> int:
        if _HAS_PSUTIL:
            try:
                c = psutil.cpu_count(logical=True)
                if c:
                    return c
            except Exception:
                pass
        try:
            c = os.cpu_count()
            if c:
                return c
        except Exception:
            pass
        try:
            txt = Path("/proc/cpuinfo").read_text(errors="ignore")
            cnt = len(re.findall(r"^processor\s*:", txt, re.MULTILINE))
            if cnt:
                return cnt
        except Exception:
            pass
        out = _run("grep -c ^processor /proc/cpuinfo 2>/dev/null")
        try:
            return int(out) if out else 1
        except Exception:
            return 1

    def get_cpu_percent(self) -> float:
        if _HAS_PSUTIL:
            try:
                return round(float(psutil.cpu_percent(interval=0.4)), 2)
            except Exception:
                pass
        # fallback /proc/stat double sample
        def _read():
            try:
                with open("/proc/stat") as f:
                    return f.read()
            except Exception:
                return ""
        s1 = _read()
        time.sleep(0.4)
        s2 = _read()
        if not s1 or not s2:
            return 0.0
        lines1 = s1.strip().splitlines()
        lines2 = s2.strip().splitlines()
        total = 0.0
        count = 0
        for l1 in lines1:
            if not l1.startswith("cpu") or l1.startswith("cpu "):
                if l1.startswith("cpu "):
                    # aggregate line, skip for per-core avg? we want per-core avg
                    continue
                if not l1.startswith("cpu"):
                    continue
                # we have cpuN lines
            if l1.startswith("cpu") and not l1[3:4].isdigit():
                # skip aggregate 'cpu '
                continue
            # find matching line in s2
            # locate by prefix
            prefix = l1.split()[0]
            l2 = next((x for x in lines2 if x.startswith(prefix + " ")), None)
            if not l2:
                continue
            p1 = l1.split()
            p2 = l2.split()
            try:
                v1 = list(map(int, p1[1:]))
                v2 = list(map(int, p2[1:]))
            except Exception:
                continue
            tot1 = sum(v1)
            tot2 = sum(v2)
            idle1 = v1[3] if len(v1) > 3 else 0
            idle2 = v2[3] if len(v2) > 3 else 0
            # include iowait
            if len(v1) > 4:
                idle1 += v1[4]
                idle2 += v2[4]
            d_tot = tot2 - tot1
            d_idle = idle2 - idle1
            if d_tot > 0:
                usage = (d_tot - d_idle) / d_tot * 100
                total += usage
                count += 1
        if count:
            return round(total / count, 2)
        # fallback single aggregate
        try:
            a1 = lines1[0].split()
            a2 = next((x for x in lines2 if x.startswith("cpu ")), "").split()
            if a1 and a2:
                v1 = list(map(int, a1[1:]))
                v2 = list(map(int, a2[1:]))
                tot1, tot2 = sum(v1), sum(v2)
                idle1 = v1[3] + (v1[4] if len(v1) > 4 else 0)
                idle2 = v2[3] + (v2[4] if len(v2) > 4 else 0)
                d_tot = tot2 - tot1
                d_idle = idle2 - idle1
                if d_tot > 0:
                    return round((d_tot - d_idle) / d_tot * 100, 2)
        except Exception:
            pass
        return 0.0

    def get_cpu_percent_per_core(self) -> list[float]:
        if _HAS_PSUTIL:
            try:
                vals = psutil.cpu_percent(interval=0.4, percpu=True)
                return [round(float(v), 2) for v in vals]
            except Exception:
                pass
        # manual per-core delta with static cache
        global _cpu_last_per_core
        try:
            lines = Path("/proc/stat").read_text(errors="ignore").splitlines()
        except Exception:
            out = _run("cat /proc/stat 2>/dev/null")
            lines = out.splitlines() if out else []
        res: list[float] = []
        now = {}
        for line in lines:
            m = re.match(r"^cpu(\d+)\s+(.+)$", line)
            if not m:
                continue
            core = int(m.group(1))
            parts = list(map(int, re.split(r"\s+", m.group(2).strip())))
            total = sum(parts)
            idle = parts[3] if len(parts) > 3 else 0
            if len(parts) > 4:
                idle += parts[4]
            prev = _cpu_last_per_core.get(core)
            now[core] = {"total": total, "idle": idle}
            if prev is None:
                res.append(0.0)
            else:
                d_tot = total - prev["total"]
                d_idle = idle - prev["idle"]
                usage = (d_tot - d_idle) / d_tot * 100 if d_tot > 0 else 0
                res.append(round(max(0, min(100, usage)), 2))
        _cpu_last_per_core = now
        # ensure sorted by core idx
        # res already in order of appearance, which is sorted
        return res

    def get_memory(self) -> dict[str, Any]:
        if _HAS_PSUTIL:
            try:
                vm = psutil.virtual_memory()
                return {
                    "total_bytes": vm.total,
                    "available_bytes": vm.available,
                    "used_bytes": vm.used,
                    "free_bytes": vm.free,
                    "percent": round(vm.percent, 1),
                    "buffers": getattr(vm, "buffers", 0),
                    "cached": getattr(vm, "cached", 0),
                    "raw": f"MemTotal: {vm.total} Available: {vm.available}",
                }
            except Exception:
                pass
        # parse /proc/meminfo
        try:
            txt = Path("/proc/meminfo").read_text(errors="ignore")
            vals: dict[str, int] = {}
            for line in txt.splitlines():
                m = re.match(r"^(\w+):\s+(\d+)\s+kB", line)
                if m:
                    vals[m.group(1)] = int(m.group(2)) * 1024
            total = vals.get("MemTotal", 0)
            avail = vals.get("MemAvailable", vals.get("MemFree", 0))
            free = vals.get("MemFree", 0)
            buffers = vals.get("Buffers", 0)
            cached = vals.get("Cached", 0)
            # for older kernels without MemAvailable
            if "MemAvailable" not in vals:
                avail = free + buffers + cached
            used = total - avail if total else 0
            percent = round(used / total * 100, 1) if total else 0
            # also run free -h for raw
            raw = _run("free -h 2>/dev/null")
            return {
                "total_bytes": total,
                "available_bytes": avail,
                "used_bytes": used,
                "free_bytes": free,
                "percent": percent,
                "buffers": buffers,
                "cached": cached,
                "raw": raw or txt[:800],
            }
        except Exception as e:
            return {"total_bytes": 0, "available_bytes": 0, "used_bytes": 0, "free_bytes": 0, "percent": 0, "raw": str(e)}

    def get_disk(self) -> list[dict[str, Any]]:
        if _HAS_PSUTIL:
            try:
                parts = psutil.disk_partitions(all=False)
                out = []
                for p in parts:
                    try:
                        u = psutil.disk_usage(p.mountpoint)
                        out.append({
                            "device": p.device,
                            "mount": p.mountpoint,
                            "fstype": p.fstype,
                            "size_bytes": u.total,
                            "used_bytes": u.used,
                            "avail_bytes": u.free,
                            "use_percent": round(u.percent, 1),
                            "size": _parse_human_bytes(u.total),
                            "used": _parse_human_bytes(u.used),
                            "avail": _parse_human_bytes(u.free),
                        })
                    except Exception:
                        continue
                if out:
                    return out
            except Exception:
                pass
        # df -hP
        txt = _run("df -hP 2>/dev/null")
        if not txt:
            # fallback single root
            try:
                u = shutil.disk_usage("/")
                return [{
                    "device": "root",
                    "mount": "/",
                    "fstype": "unknown",
                    "size_bytes": u.total,
                    "used_bytes": u.used,
                    "avail_bytes": u.free,
                    "use_percent": round(u.used / u.total * 100, 1) if u.total else 0,
                    "size": _parse_human_bytes(u.total),
                    "used": _parse_human_bytes(u.used),
                    "avail": _parse_human_bytes(u.free),
                }]
            except Exception:
                return []
        lines = txt.strip().splitlines()
        res = []
        for line in lines[1:]:
            parts = re.split(r"\s+", line.strip())
            if len(parts) < 6:
                continue
            dev, size, used, avail, pct, mnt = parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]
            # also get bytes via df -B1 if needed
            # parse percent
            try:
                use_pct = float(pct.strip("%"))
            except Exception:
                use_pct = 0
            # try to get bytes via shutil for that mount if accessible
            size_b = used_b = avail_b = 0
            try:
                u = shutil.disk_usage(mnt)
                size_b, used_b, avail_b = u.total, u.used, u.free
            except Exception:
                # keep from df -h parsing?
                pass
            res.append({
                "device": dev,
                "mount": mnt,
                "fstype": "unknown",
                "size": size,
                "used": used,
                "avail": avail,
                "use_percent": use_pct,
                "size_bytes": size_b,
                "used_bytes": used_b,
                "avail_bytes": avail_b,
                "raw": line,
            })
        return res

    def get_uptime(self) -> str:
        # /proc/uptime first
        try:
            txt = Path("/proc/uptime").read_text(errors="ignore").strip().split()
            if txt:
                sec = float(txt[0])
                human = _uptime_human(sec)
                # also include load style
                out = _run("uptime 2>/dev/null")
                return out or f"up {human}"
        except Exception:
            pass
        return super().get_uptime()

    def get_uptime_seconds(self) -> float | None:
        try:
            txt = Path("/proc/uptime").read_text(errors="ignore").strip().split()
            if txt:
                return float(txt[0])
        except Exception:
            pass
        return None


class BsdSystemInfo(BaseSystemInfo):
    """Covers FreeBSD, OpenBSD, NetBSD, Darwin (similar sysctl)."""

    def _sysctl(self, key: str) -> str:
        return _run(f"sysctl -n {key} 2>/dev/null")

    def get_cpu_cores(self) -> int:
        if _HAS_PSUTIL:
            try:
                c = psutil.cpu_count(logical=True)
                if c:
                    return c
            except Exception:
                pass
        try:
            c = os.cpu_count()
            if c:
                return c
        except Exception:
            pass
        out = self._sysctl("hw.ncpu")
        try:
            n = int(out.strip())
            if n > 0:
                return n
        except Exception:
            pass
        # fallback parse dmesg
        out2 = _run("dmesg | grep -i 'cpu' | head -n1 2>/dev/null")
        m = re.search(r"(\d+)\s*CPU", out2, re.I)
        if m:
            try:
                return int(m.group(1))
            except Exception:
                pass
        return 1

    def get_cpu_percent(self) -> float:
        if _HAS_PSUTIL:
            try:
                return round(float(psutil.cpu_percent(interval=0.4)), 2)
            except Exception:
                pass
        # BSD kern.cp_time: user nice sys intr idle
        def _read():
            raw = self._sysctl("kern.cp_time")
            if not raw:
                return None
            parts = list(map(int, re.split(r"\s+", raw.strip())))
            if len(parts) < 5:
                return None
            return {"user": parts[0], "nice": parts[1], "sys": parts[2], "intr": parts[3], "idle": parts[4], "total": sum(parts[:5])}
        cur = _read()
        if cur is None:
            return 0.0
        global _cpu_last
        prev = _cpu_last.get("bsd_total")
        if prev is None:
            _cpu_last["bsd_total"] = cur
            time.sleep(0.35)
            cur2 = _read()
            if cur2 is None:
                return 0.0
            prev = _cpu_last["bsd_total"]
            cur = cur2
            _cpu_last["bsd_total"] = cur
            d_tot = cur["total"] - prev["total"]
            d_idle = cur["idle"] - prev["idle"]
            if d_tot <= 0:
                return 0.0
            usage = (d_tot - d_idle) / d_tot * 100
            return round(max(0, min(100, usage)), 2)
        else:
            _cpu_last["bsd_total"] = cur
            d_tot = cur["total"] - prev["total"]
            d_idle = cur["idle"] - prev["idle"]
            if d_tot <= 0:
                return 0.0
            usage = (d_tot - d_idle) / d_tot * 100
            return round(max(0, min(100, usage)), 2)

    def get_cpu_percent_per_core(self) -> list[float]:
        if _HAS_PSUTIL:
            try:
                vals = psutil.cpu_percent(interval=0.4, percpu=True)
                return [round(float(v), 2) for v in vals]
            except Exception:
                pass
        raw = self._sysctl("kern.cp_times")
        if not raw:
            # per-core not available, distribute total
            total = self.get_cpu_percent()
            cores = self.get_cpu_cores()
            # return per-core as equal slices with small variation disabled
            # need to avoid recursion infinite: just return [total]*cores with 0 on first?
            # Better to return single value repeated
            # But to avoid calling get_cpu_percent again which updates cache, just return []
            # Provide empty to indicate not available, UI will fallback to total
            # For better UX, return total repeated
            return [round(total, 2)] * cores if cores else [0.0]

        parts = list(map(int, re.split(r"\s+", raw.strip())))
        # parts is flat array len = cores*5
        cores = len(parts) // 5
        res = []
        global _cpu_last
        # use dict per core stored in _cpu_last
        last_per = _cpu_last.get("bsd_per_core", {})
        new_last: dict[int, dict[str, int]] = {}
        for i in range(cores):
            off = i * 5
            user, nice, sysv, intr, idle = parts[off:off+5]
            total = user + nice + sysv + intr + idle
            prev = last_per.get(i)
            new_last[i] = {"total": total, "idle": idle}
            if prev is None:
                res.append(0.0)
            else:
                d_tot = total - prev["total"]
                d_idle = idle - prev["idle"]
                usage = (d_tot - d_idle) / d_tot * 100 if d_tot > 0 else 0
                res.append(round(max(0, min(100, usage)), 2))
        _cpu_last["bsd_per_core"] = new_last
        return res

    def get_memory(self) -> dict[str, Any]:
        if _HAS_PSUTIL:
            try:
                vm = psutil.virtual_memory()
                return {
                    "total_bytes": vm.total,
                    "available_bytes": vm.available,
                    "used_bytes": vm.used,
                    "free_bytes": vm.free,
                    "percent": round(vm.percent, 1),
                    "raw": f"total {vm.total} free {vm.free}",
                }
            except Exception:
                pass
        # Try hw.physmem
        total_s = self._sysctl("hw.physmem")
        if not total_s:
            total_s = self._sysctl("hw.usermem")
        try:
            total = int(total_s.strip()) if total_s else 0
        except Exception:
            total = 0
        # Try to get page stats
        # FreeBSD: vm.stats.vm.v_page_count etc
        # OpenBSD: similar but keys differ
        page_size_s = self._sysctl("hw.pagesize") or self._sysctl("vm.stats.vm.v_page_size")
        try:
            page_size = int(page_size_s.strip()) if page_size_s else 4096
        except Exception:
            page_size = 4096
        # Try vmtotal for FreeBSD raw
        vmtotal = _run("sysctl vm.vmtotal 2>/dev/null")
        raw = vmtotal or _run("sysctl hw.physmem vm.stats.vm.v_page_size 2>/dev/null") or f"hw.physmem {total}"
        # Try vm stats for free
        free = 0
        avail = 0
        # Attempt to get free pages
        for key in ["vm.stats.vm.v_free_count", "vm.stats.vm.v_cache_count", "vm.stats.vm.v_inactive_count"]:
            val = self._sysctl(key)
            try:
                if val:
                    free += int(val.strip()) * page_size
            except Exception:
                pass
        # For OpenBSD, use vmstat?
        if not free:
            # try top output parsing
            top_out = _run("top -l 1 -n 0 2>/dev/null | head -n 10") or _run("top -b -n1 2>/dev/null | head -n 20")
            # Try to parse PhysMem
            m = re.search(r"PhysMem:\s*([\d\.]+)([GMK])\s*used.*?([\d\.]+)([GMK])\s*free", top_out)
            if m:
                def _to_b(v, u):
                    mult = {"G": 1024**3, "M": 1024**2, "K": 1024}.get(u, 1)
                    return float(v) * mult
                try:
                    used = _to_b(m.group(1), m.group(2))
                    free = _to_b(m.group(3), m.group(4))
                    total = int(used + free) if total == 0 else total
                    avail = free
                    percent = round(used / (used+free) * 100, 1) if (used+free) else 0
                    return {"total_bytes": int(total), "available_bytes": int(avail), "used_bytes": int(used), "free_bytes": int(free), "percent": percent, "raw": top_out[:800]}
                except Exception:
                    pass
        if total and free:
            avail = free
            used = total - avail
            percent = round(used / total * 100, 1) if total else 0
            return {"total_bytes": total, "available_bytes": avail, "used_bytes": used, "free_bytes": free, "percent": percent, "raw": raw[:800]}
        # Final fallback: use total only
        if total:
            # Assume 70% used if unknown free
            return {"total_bytes": total, "available_bytes": 0, "used_bytes": total, "free_bytes": 0, "percent": 0, "raw": raw[:800]}
        return {"total_bytes": 0, "available_bytes": 0, "used_bytes": 0, "free_bytes": 0, "percent": 0, "raw": raw[:800] if raw else "unknown"}

    def get_disk(self) -> list[dict[str, Any]]:
        if _HAS_PSUTIL:
            try:
                parts = psutil.disk_partitions(all=False)
                out = []
                for p in parts:
                    try:
                        u = psutil.disk_usage(p.mountpoint)
                        out.append({
                            "device": p.device,
                            "mount": p.mountpoint,
                            "fstype": p.fstype,
                            "size_bytes": u.total,
                            "used_bytes": u.used,
                            "avail_bytes": u.free,
                            "use_percent": round(u.percent, 1),
                            "size": _parse_human_bytes(u.total),
                            "used": _parse_human_bytes(u.used),
                            "avail": _parse_human_bytes(u.free),
                        })
                    except Exception:
                        continue
                if out:
                    return out
            except Exception:
                pass
        txt = _run("df -hP 2>/dev/null") or _run("df -h 2>/dev/null")
        if not txt:
            try:
                u = shutil.disk_usage("/")
                return [{
                    "device": "root",
                    "mount": "/",
                    "fstype": "unknown",
                    "size_bytes": u.total,
                    "used_bytes": u.used,
                    "avail_bytes": u.free,
                    "use_percent": round(u.used / u.total * 100, 1) if u.total else 0,
                    "size": _parse_human_bytes(u.total),
                    "used": _parse_human_bytes(u.used),
                    "avail": _parse_human_bytes(u.free),
                }]
            except Exception:
                return []
        lines = txt.strip().splitlines()
        res = []
        for line in lines[1:]:
            parts = re.split(r"\s+", line.strip())
            if len(parts) < 6:
                # BSD df may have wrapped lines; skip
                continue
            # Try to handle device with spaces? Assume last 5 are size used avail cap mount
            # So device is join of remaining
            if len(parts) > 6:
                # e.g. /dev/mapper/... with spaces unlikely
                dev = " ".join(parts[:-5])
                size, used, avail, cap, mnt = parts[-5], parts[-4], parts[-3], parts[-2], parts[-1]
            else:
                dev, size, used, avail, cap, mnt = parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]
            try:
                pct = float(cap.strip("%"))
            except Exception:
                pct = 0
            size_b = used_b = avail_b = 0
            try:
                u = shutil.disk_usage(mnt)
                size_b, used_b, avail_b = u.total, u.used, u.free
            except Exception:
                pass
            res.append({
                "device": dev,
                "mount": mnt,
                "fstype": "unknown",
                "size": size,
                "used": used,
                "avail": avail,
                "use_percent": pct,
                "size_bytes": size_b,
                "used_bytes": used_b,
                "avail_bytes": avail_b,
                "raw": line,
            })
        return res

    def get_uptime(self) -> str:
        # Try kern.boottime
        bt = self._sysctl("kern.boottime")
        # Format like "{ sec = 123456789, usec = 0 } Tue May ..."
        m = re.search(r"sec\s*=\s*(\d+)", bt)
        if m:
            try:
                sec = int(m.group(1))
                now = int(time.time())
                diff = now - sec
                return f"up {_uptime_human(diff)}"
            except Exception:
                pass
        # fallback
        out = _run("uptime 2>/dev/null")
        return out or super().get_uptime()

    def get_uptime_seconds(self) -> float | None:
        bt = self._sysctl("kern.boottime")
        m = re.search(r"sec\s*=\s*(\d+)", bt)
        if m:
            try:
                sec = int(m.group(1))
                return float(int(time.time()) - sec)
            except Exception:
                pass
        return None


class GenericSystemInfo(BaseSystemInfo):
    # Fallback using only psutil + generic commands, for Darwin/Windows
    def get_cpu_cores(self) -> int:
        if _HAS_PSUTIL:
            try:
                c = psutil.cpu_count(logical=True)  # type: ignore
                if c:
                    return c
            except Exception:
                pass
        return os.cpu_count() or 1

    def get_cpu_percent(self) -> float:
        if _HAS_PSUTIL:
            try:
                return round(float(psutil.cpu_percent(interval=0.4)), 2)  # type: ignore
            except Exception:
                pass
        return 0.0

    def get_cpu_percent_per_core(self) -> list[float]:
        if _HAS_PSUTIL:
            try:
                vals = psutil.cpu_percent(interval=0.4, percpu=True)  # type: ignore
                return [round(float(v), 2) for v in vals]
            except Exception:
                pass
        return []

    def get_memory(self) -> dict[str, Any]:
        if _HAS_PSUTIL:
            try:
                vm = psutil.virtual_memory()  # type: ignore
                return {
                    "total_bytes": vm.total,
                    "available_bytes": vm.available,
                    "used_bytes": vm.used,
                    "free_bytes": vm.free,
                    "percent": round(vm.percent, 1),
                    "raw": "",
                }
            except Exception:
                pass
        # try free or vm_stat
        raw = _run("free -h 2>/dev/null") or _run("vm_stat 2>/dev/null | head -n 20")
        return {"total_bytes": 0, "available_bytes": 0, "used_bytes": 0, "free_bytes": 0, "percent": 0, "raw": raw[:800]}

    def get_disk(self) -> list[dict[str, Any]]:
        if _HAS_PSUTIL:
            try:
                parts = psutil.disk_partitions(all=False)  # type: ignore
                out = []
                for p in parts:
                    try:
                        u = psutil.disk_usage(p.mountpoint)  # type: ignore
                        out.append({
                            "device": p.device,
                            "mount": p.mountpoint,
                            "fstype": p.fstype,
                            "size_bytes": u.total,
                            "used_bytes": u.used,
                            "avail_bytes": u.free,
                            "use_percent": round(u.percent, 1),
                            "size": _parse_human_bytes(u.total),
                            "used": _parse_human_bytes(u.used),
                            "avail": _parse_human_bytes(u.free),
                        })
                    except Exception:
                        continue
                if out:
                    return out
            except Exception:
                pass
        # fallback df
        txt = _run("df -hP 2>/dev/null") or _run("df -h 2>/dev/null")
        if txt:
            lines = txt.strip().splitlines()
            res = []
            for line in lines[1:]:
                parts = re.split(r"\s+", line.strip())
                if len(parts) >= 6:
                    dev, size, used, avail, cap, mnt = parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]
                    try:
                        pct = float(cap.strip("%"))
                    except Exception:
                        pct = 0
                    res.append({"device": dev, "mount": mnt, "size": size, "used": used, "avail": avail, "use_percent": pct, "size_bytes": 0, "used_bytes": 0, "avail_bytes": 0, "fstype": "unknown"})
            return res
        try:
            u = shutil.disk_usage("/")
            return [{"device": "root", "mount": "/", "fstype": "unknown", "size_bytes": u.total, "used_bytes": u.used, "avail_bytes": u.free, "use_percent": round(u.used / u.total * 100, 1) if u.total else 0, "size": _parse_human_bytes(u.total), "used": _parse_human_bytes(u.used), "avail": _parse_human_bytes(u.free)}]
        except Exception:
            return []


def get_system_info() -> BaseSystemInfo:
    sysname = platform.system()
    # Also check platform.platform() lower for bsd detection via uname
    lower = sysname.lower()
    if lower == "linux":
        return LinuxSystemInfo()
    elif lower in ("freebsd", "openbsd", "netbsd", "bsd"):
        return BsdSystemInfo()
    elif "bsd" in lower:  # e.g. "FreeBSD" already, but handle "Darwin" separately
        return BsdSystemInfo()
    elif lower == "darwin":
        # Darwin shares BSD sysctl semantics partly; use BsdSystemInfo for cpu/mem
        return BsdSystemInfo()
    else:
        # Windows or unknown
        return GenericSystemInfo()


# singleton helper
_system_instance: BaseSystemInfo | None = None

def get_instance() -> BaseSystemInfo:
    global _system_instance
    if _system_instance is None:
        _system_instance = get_system_info()
    return _system_instance

# For testing, allow reset
def _reset():
    global _system_instance, _cpu_last, _cpu_last_per_core, _last_global
    _system_instance = None
    _cpu_last = {}
    _cpu_last_per_core = {}
    _last_global = {}
