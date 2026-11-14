"""Build a checked portable archive from an explicit, already-tested Windows staging folder.

Python 3.11+, standard library only. Does not install, sign, publish or copy user data.
"""
from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
from pathlib import Path, PurePosixPath
import struct
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parent.parent
BINARIES = ("MuseFlow-PC.exe", "libmpv-2.dll", "resolver/yt-dlp.exe", "javascript/deno.exe")
DOCUMENTS = ("README.txt", "LICENSE", "THIRD_PARTY_NOTICES.md", "CHANGELOG.md", "TRAY.md",
             "DESKTOP-FEATURES.md", "LOCAL-PLAYER.md", "ONLINE-PLAYER.md", "VALIDATION.md",
             "dependency-inventory.json", "mpv-provenance.json", "runtime-info.json",
             "resolver-provenance.json", "SHA256SUMS.txt", "AUDIO-OUTPUT.md", "SIMPMUSIC-RESEARCH.md", "ARTWORK.md")


def digest(stream) -> str:
    h = hashlib.sha256()
    for block in iter(lambda: stream.read(1024 * 1024), b""):
        h.update(block)
    return h.hexdigest()


def file_version(exe: Path) -> str:
    """Read the actual PE fixed-version resource without executing the binary."""
    api = ctypes.WinDLL("version", use_last_error=True)
    api.GetFileVersionInfoSizeW.argtypes = [wintypes.LPCWSTR, ctypes.POINTER(wintypes.DWORD)]
    api.GetFileVersionInfoSizeW.restype = wintypes.DWORD
    api.GetFileVersionInfoW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p]
    api.VerQueryValueW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR, ctypes.POINTER(ctypes.c_void_p), ctypes.POINTER(wintypes.UINT)]
    size = api.GetFileVersionInfoSizeW(str(exe), None)
    if not size:
        raise ValueError("Executable has no Windows version resource")
    buffer = ctypes.create_string_buffer(size)
    if not api.GetFileVersionInfoW(str(exe), 0, size, buffer):
        raise ctypes.WinError(ctypes.get_last_error())
    pointer, length = ctypes.c_void_p(), wintypes.UINT()
    if not api.VerQueryValueW(buffer, "\\", ctypes.byref(pointer), ctypes.byref(length)) or length.value < 52:
        raise ValueError("Invalid fixed-version resource")
    values = struct.unpack("<13I", ctypes.string_at(pointer, 52))
    if values[0] != 0xFEEF04BD:
        raise ValueError("Invalid version-resource signature")
    major_minor, patch_build = values[2:4]
    return f"{major_minor >> 16}.{major_minor & 65535}.{patch_build >> 16}"


def package(staging: Path, output: Path) -> None:
    staging = staging.resolve(strict=True)
    output = output.resolve()
    version = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
    if file_version(staging / BINARIES[0]) != version.split("-", 1)[0]:
        raise ValueError("Executable version differs from Cargo.toml")
    with (staging / BINARIES[0]).open("rb") as stream:
        header = stream.read(64)
        if header[:2] != b"MZ":
            raise ValueError("Input is not a Windows executable")
        stream.seek(struct.unpack_from("<I", header, 60)[0])
        if stream.read(6) != b"PE\0\0\x64\x86":
            raise ValueError("Portable package requires a Windows x64 executable")
    if output.exists():
        raise FileExistsError(f"Refusing to overwrite {output}")
    files = {name: staging / name for name in (*BINARIES, *DOCUMENTS)}
    license_root = staging / "licenses"
    if not license_root.is_dir():
        raise ValueError("Native runtime license folder is required")
    files.update({p.relative_to(staging).as_posix(): p for p in license_root.rglob("*") if p.is_file()})
    for name, path in files.items():
        if not path.is_file() or path.is_symlink() or not path.resolve().is_relative_to(staging):
            raise ValueError(f"Missing or external package input: {name}")
    expected = {}
    for line in (staging / "SHA256SUMS.txt").read_text(encoding="utf-8").splitlines():
        h, name = line.split(maxsplit=1)
        expected[name.strip()] = h.lower()
    for name in BINARIES:
        with files[name].open("rb") as stream:
            if digest(stream) != expected.get(name):
                raise ValueError(f"Staging binary checksum mismatch: {name}")
    prefix = f"MuseFlow-PC-{version}-windows-x64"
    output.parent.mkdir(parents=True, exist_ok=True)
    records = []
    try:
        with zipfile.ZipFile(output, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
            # Explicit allowlist excludes source, caches, data, SQLite, local settings and keys.
            for name, path in sorted(files.items()):
                if name == "resolver-provenance.json":
                    raw = json.dumps([{k: v for k, v in item.items() if k != "path"}
                                      for item in json.loads(path.read_text(encoding="utf-8"))], indent=2).encode()
                    archive.writestr(prefix + "/" + name, raw)
                    h, size = hashlib.sha256(raw).hexdigest(), len(raw)
                else:
                    archive.write(path, prefix + "/" + name)
                    with path.open("rb") as stream:
                        h = digest(stream)
                    size = path.stat().st_size
                records.append({"path": name, "bytes": size, "sha256": h})
            marker = b"data\n"
            archive.writestr(prefix + "/portable-data-dir.txt", marker)
            records.append({"path": "portable-data-dir.txt", "bytes": len(marker), "sha256": hashlib.sha256(marker).hexdigest()})
            readme = (
                             "Extract this entire folder to your SSD and run MuseFlow-PC.exe.\n"
                             "The portable copy stores library/settings in its adjacent data/ folder.\n"
                             "On the development PC, keep installation/extraction under G:\\Essential Softwares.\n"
                             "Close exits by default. Settings can enable Close window to tray.\n"
                             "Right-click the tray icon for Exit MuseFlow.\n"
                             "Unsigned local preview; see VALIDATION.md for remaining platform/distribution checks.\n").encode("utf-8")
            archive.writestr(prefix + "/PORTABLE-README.txt", readme)
            records.append({"path": "PORTABLE-README.txt", "bytes": len(readme), "sha256": hashlib.sha256(readme).hexdigest()})
            manifest = {"format_version": 1, "version": version, "architecture": "windows-x64", "files": records}
            archive.writestr(prefix + "/PORTABLE-MANIFEST.json", json.dumps(manifest, indent=2))
        with zipfile.ZipFile(output) as archive:
            bad = archive.testzip()
            if bad:
                raise ValueError(f"Archive CRC failure: {bad}")
            for name in archive.namelist():
                path = PurePosixPath(name)
                if path.is_absolute() or ".." in path.parts or path.parts[0] != prefix:
                    raise ValueError("Invalid archive member path")
            for record in records:
                with archive.open(prefix + "/" + record["path"]) as stream:
                    if digest(stream) != record["sha256"]:
                        raise ValueError("Portable archive content hash mismatch")
        with output.open("rb") as stream:
            h = digest(stream)
        output.with_suffix(output.suffix + ".sha256").write_text(h + "  " + output.name + "\n", encoding="utf-8")
        print(json.dumps({"archive": str(output), "bytes": output.stat().st_size, "sha256": h, "version": version}))
    except Exception:
        # Leave a failed output for inspection; never silently replace or publish it.
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--staging", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args()
    package(options.staging, options.output)
