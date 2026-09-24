#!/usr/bin/env python3
"""
Tomb of Heroes Version Bumping Script.

Manages SemVer 2.0 version progression across workspace files.
Strictly enforces:
1. Major version MUST remain 0 during conception (ZeroVer: 0.MINOR.PATCH).
2. Minor revisions advance when substantial milestones or systems are integrated.
3. Patch revisions advance for targeted fixes or refinements within a milestone.

Usage:
    python3 scripts/bump_version.py --type minor
    python3 scripts/bump_version.py --type patch
    python3 scripts/bump_version.py --set 0.2.0
    python3 scripts/bump_version.py --check
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
ROOT_CARGO = REPO_ROOT / "Cargo.toml"
APP_CARGO = REPO_ROOT / "crates" / "tomb_of_heroes_app" / "Cargo.toml"
IOS_PLIST = REPO_ROOT / "packaging" / "ios" / "Info.plist"


def get_current_version() -> tuple[int, int, int]:
    content = ROOT_CARGO.read_text(encoding="utf-8")
    m = re.search(r'\[workspace\.package\][\s\S]*?version\s*=\s*"(\d+)\.(\d+)\.(\d+)"', content)
    if not m:
        raise ValueError(f"Could not parse valid SemVer version from {ROOT_CARGO}")
    return int(m.group(1)), int(m.group(2)), int(m.group(3))


def format_version(major: int, minor: int, patch: int) -> str:
    return f"{major}.{minor}.{patch}"


def update_root_cargo(new_version: str) -> None:
    content = ROOT_CARGO.read_text(encoding="utf-8")
    content = re.sub(
        r'(\[workspace\.package\][\s\S]*?version\s*=\s*)"[^"]*"',
        rf'\g<1>"{new_version}"',
        content,
        count=1
    )
    ROOT_CARGO.write_text(content, encoding="utf-8")
    print(f"  • Updated {ROOT_CARGO} -> version = \"{new_version}\"")


def update_app_cargo(new_version: str) -> None:
    content = APP_CARGO.read_text(encoding="utf-8")
    if re.search(r'version_name\s*=', content):
        content = re.sub(
            r'version_name\s*=\s*"[^"]*"',
            f'version_name = "{new_version}"',
            content
        )
        APP_CARGO.write_text(content, encoding="utf-8")
        print(f"  • Updated {APP_CARGO} -> version_name = \"{new_version}\"")


def update_ios_plist(new_version: str) -> None:
    content = IOS_PLIST.read_text(encoding="utf-8")
    content = re.sub(
        r'(<key>CFBundleShortVersionString</key>\s*<string>)[^<]*(</string>)',
        rf'\g<1>{new_version}\g<2>',
        content
    )
    IOS_PLIST.write_text(content, encoding="utf-8")
    print(f"  • Updated {IOS_PLIST} -> CFBundleShortVersionString = {new_version}")


def refresh_cargo_lock() -> None:
    print("  • Updating Cargo.lock...")
    cargo_bin = "cargo"
    try:
        subprocess.run(
            [cargo_bin, "check", "--workspace", "--quiet"],
            cwd=REPO_ROOT,
            check=True
        )
        print("  • Cargo.lock refreshed successfully.")
    except (subprocess.CalledProcessError, FileNotFoundError) as e:
        print(f"  [WARN] Failed to automatically run 'cargo check' to update Cargo.lock: {e}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Tomb of Heroes version manager.")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--type", choices=["minor", "patch"], help="Bump type (minor or patch)")
    group.add_argument("--set", type=str, help="Explicit SemVer version (must be 0.Y.Z)")
    group.add_argument("--check", action="store_true", help="Check current version consistency")

    args = parser.parse_args()

    cur_major, cur_minor, cur_patch = get_current_version()
    current_str = format_version(cur_major, cur_minor, cur_patch)

    if args.check:
        print(f"Current workspace version: {current_str}")
        assert cur_major == 0, f"Major version MUST be 0 during conception, found {cur_major}"
        print("✓ Version consistency checks passed.")
        return 0

    if args.set:
        m = re.match(r"^(\d+)\.(\d+)\.(\d+)$", args.set.strip())
        if not m:
            print(f"Error: Version '{args.set}' does not match SemVer format 'X.Y.Z'", file=sys.stderr)
            return 1
        new_major, new_minor, new_patch = int(m.group(1)), int(m.group(2)), int(m.group(3))
    elif args.type == "minor":
        new_major = cur_major
        new_minor = cur_minor + 1
        new_patch = 0
    elif args.type == "patch":
        new_major = cur_major
        new_minor = cur_minor
        new_patch = cur_patch + 1
    else:
        print("Error: Unknown bump action", file=sys.stderr)
        return 1

    # Invariant enforcement: Major version MUST remain 0 during conception
    if new_major != 0:
        print(
            f"Error: Conception phase invariant violated! Major version must remain 0 (attempted: {new_major}).",
            file=sys.stderr
        )
        print(
            "Refer to docs/VERSIONING.md for policy on major release promotions.",
            file=sys.stderr
        )
        return 1

    new_str = format_version(new_major, new_minor, new_patch)
    print(f"Bumping version: {current_str} -> {new_str}")

    update_root_cargo(new_str)
    update_app_cargo(new_str)
    update_ios_plist(new_str)
    refresh_cargo_lock()

    print(f"\n✓ Successfully bumped version to {new_str}")
    print(f"Suggested commit message: chore(release): bump version to {new_str}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
