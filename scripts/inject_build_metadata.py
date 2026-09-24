#!/usr/bin/env python3
"""
Inject build number and metadata into Android and iOS configurations.

Usage:
    python3 scripts/inject_build_metadata.py --build-number 42 [--sha d819447]
"""

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent


def get_workspace_version(cargo_toml_path: Path) -> str:
    content = cargo_toml_path.read_text(encoding="utf-8")
    m = re.search(r'\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"', content)
    if not m:
        raise ValueError(f"Could not find [workspace.package] version in {cargo_toml_path}")
    return m.group(1)


def update_android_manifest(app_cargo_toml_path: Path, build_num: int, full_version: str) -> None:
    content = app_cargo_toml_path.read_text(encoding="utf-8")
    
    # Check if version_code already exists under [package.metadata.android]
    if re.search(r'version_code\s*=', content):
        content = re.sub(r'version_code\s*=\s*\d+', f'version_code = {build_num}', content)
    else:
        content = re.sub(
            r'(\[package\.metadata\.android\])',
            f'\\1\nversion_code = {build_num}',
            content
        )

    # Check if version_name already exists under [package.metadata.android]
    if re.search(r'version_name\s*=', content):
        content = re.sub(r'version_name\s*=\s*"[^"]*"', f'version_name = "{full_version}"', content)
    else:
        content = re.sub(
            r'(\[package\.metadata\.android\])',
            f'\\1\nversion_name = "{full_version}"',
            content
        )

    app_cargo_toml_path.write_text(content, encoding="utf-8")
    print(f"Updated Android metadata in {app_cargo_toml_path}: versionCode={build_num}, versionName={full_version}")


def update_ios_plist(plist_path: Path, build_num: int, base_version: str) -> None:
    content = plist_path.read_text(encoding="utf-8")

    # CFBundleVersion (Build number)
    content = re.sub(
        r'(<key>CFBundleVersion</key>\s*<string>)[^<]*(</string>)',
        rf'\g<1>{build_num}\g<2>',
        content
    )

    # CFBundleShortVersionString (Marketing / SemVer)
    content = re.sub(
        r'(<key>CFBundleShortVersionString</key>\s*<string>)[^<]*(</string>)',
        rf'\g<1>{base_version}\g<2>',
        content
    )

    plist_path.write_text(content, encoding="utf-8")
    print(f"Updated iOS metadata in {plist_path}: CFBundleVersion={build_num}, CFBundleShortVersionString={base_version}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Inject build metadata into mobile configs.")
    parser.add_argument("--build-number", type=int, required=True, help="CI build run number (integer)")
    parser.add_argument("--sha", type=str, default="", help="Git commit SHA")
    args = parser.parse_args()

    root_cargo = REPO_ROOT / "Cargo.toml"
    app_cargo = REPO_ROOT / "crates" / "tomb_of_heroes_app" / "Cargo.toml"
    ios_plist = REPO_ROOT / "packaging" / "ios" / "Info.plist"

    base_ver = get_workspace_version(root_cargo)
    sha_suffix = f".{args.sha[:7]}" if args.sha else ""
    full_ver = f"{base_ver}+build.{args.build_number}{sha_suffix}"

    print(f"Injecting build metadata: Base={base_ver}, BuildNumber={args.build_number}, Full={full_ver}")

    update_android_manifest(app_cargo, args.build_number, full_ver)
    update_ios_plist(ios_plist, args.build_number, base_ver)

    return 0


if __name__ == "__main__":
    sys.exit(main())
