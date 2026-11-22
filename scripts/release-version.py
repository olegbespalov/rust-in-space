import argparse
import re
import subprocess
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description="Bump or check the Cargo package version.")
    parser.add_argument("action", choices=("bump", "check-tag"))
    parser.add_argument("value")
    args = parser.parse_args()

    manifest_path = Path("Cargo.toml")
    manifest = manifest_path.read_text(encoding="utf-8")
    package = re.search(r"^\[package\][ \t]*\n.*?(?=^\[|\Z)", manifest, re.MULTILINE | re.DOTALL)
    version = re.search(r'^version\s*=\s*"([^"\n]+)"', package[0], re.MULTILINE) if package else None
    if version is None:
        parser.exit(1, "Could not find [package].version in Cargo.toml.\n")

    if args.action == "check-tag":
        expected_tag = f"v{version[1]}"
        if args.value != expected_tag:
            parser.exit(1, f"Release tag {args.value} must match Cargo.toml ({expected_tag}).\n")
        return

    if not re.fullmatch(r"[0-9][0-9A-Za-z.+-]*", args.value):
        parser.exit(1, "Supply a version such as make bump-version VERSION=0.6.1.\n")

    lock_path = Path("Cargo.lock")
    lock = lock_path.read_bytes()
    start = package.start() + version.start(1)
    end = package.start() + version.end(1)
    manifest_path.write_text(manifest[:start] + args.value + manifest[end:], encoding="utf-8")
    try:
        subprocess.run(["cargo", "update", "--workspace"], check=True)
    except (OSError, subprocess.CalledProcessError, KeyboardInterrupt):
        manifest_path.write_text(manifest, encoding="utf-8")
        if not lock_path.exists() or lock_path.read_bytes() != lock:
            lock_path.write_bytes(lock)
        parser.exit(1, "Could not update Cargo.lock. Restored the previous version.\n")


if __name__ == "__main__":
    main()
