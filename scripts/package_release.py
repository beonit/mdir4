#!/usr/bin/env python3
"""Build a self-contained Mdir4 release archive from an existing binary."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import zipfile


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def cargo_metadata(root: Path) -> dict:
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        cwd=root,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return json.loads(result.stdout)


def license_inventory(metadata: dict) -> str:
    workspace = set(metadata["workspace_members"])
    packages = sorted(
        (package for package in metadata["packages"] if package["id"] not in workspace),
        key=lambda package: (package["name"].casefold(), package["version"]),
    )
    lines = [
        "Mdir4 third-party dependency license inventory",
        "",
        "Generated from Cargo metadata for the locked dependency graph.",
        "This inventory records SPDX license expressions; consult each dependency",
        "source distribution for its complete license text and notices.",
        "",
    ]
    missing = []
    for package in packages:
        license_value = package.get("license") or package.get("license_file")
        if not license_value:
            missing.append(f"{package['name']} {package['version']}")
            license_value = "NOT DECLARED"
        source = package.get("repository") or package.get("homepage") or "(not declared)"
        lines.append(
            f"- {package['name']} {package['version']} | {license_value} | {source}"
        )
    if missing:
        raise RuntimeError("dependencies without license metadata: " + ", ".join(missing))
    return "\n".join(lines) + "\n"


def syntax_acknowledgements(root: Path) -> str:
    result = subprocess.run(
        ["cargo", "run", "--quiet", "--locked", "--bin", "generate_syntax_notice"],
        cwd=root,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    if not result.stdout.strip():
        raise RuntimeError("syntax acknowledgement generator returned no content")
    return result.stdout


def archive_name(version: str, target: str) -> str:
    return f"mdir4-v{version}-{target}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--output", type=Path, default=Path("dist"))
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error(f"binary does not exist: {binary}")

    metadata = cargo_metadata(root)
    root_package = next(
        package for package in metadata["packages"] if package["id"] in metadata["workspace_members"]
    )
    stem = archive_name(root_package["version"], args.target)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    archive = output / f"{stem}.zip"

    with tempfile.TemporaryDirectory(prefix="mdir4-package-") as temporary:
        package_dir = Path(temporary) / stem
        package_dir.mkdir()
        binary_name = "mdir4.exe" if args.target.startswith("windows-") else "mdir4"
        shutil.copy2(binary, package_dir / binary_name)
        shutil.copy2(root / "README.md", package_dir / "README.md")
        shutil.copy2(root / "docs" / "development.md", package_dir / "DEVELOPMENT.md")
        (package_dir / "THIRD_PARTY_LICENSES.txt").write_text(
            license_inventory(metadata), encoding="utf-8"
        )
        (package_dir / "SYNTAX_ACKNOWLEDGEMENTS.md").write_text(
            syntax_acknowledgements(root), encoding="utf-8"
        )
        if archive.exists():
            archive.unlink()
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as bundle:
            for path in sorted(package_dir.rglob("*")):
                if path.is_file():
                    bundle.write(path, path.relative_to(package_dir.parent))

    checksum = sha256(archive)
    checksum_file = output / "SHA256SUMS"
    checksum_file.write_text(f"{checksum}  {archive.name}\n", encoding="ascii")
    print(f"Package: {archive}")
    print(f"SHA-256: {checksum}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"package_release: {error}", file=sys.stderr)
        sys.exit(1)
