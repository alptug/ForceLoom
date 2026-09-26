"""Validate inherited package versions and optionally the exact release tag."""
import argparse
import json
import subprocess
import tomllib
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--tag")
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
workspace = tomllib.loads((root / "Cargo.toml").read_text())
version = workspace["workspace"]["package"]["version"]
metadata = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=root))
for package in metadata["packages"]:
    if package["version"] != version:
        raise SystemExit(f"Version mismatch: {package['name']}")
    manifest = tomllib.loads(Path(package["manifest_path"]).read_text())
    if manifest["package"]["version"] != {"workspace": True}:
        raise SystemExit("Every package must inherit the workspace version")
project = tomllib.loads((root / "pyproject.toml").read_text())["project"]
if "version" in project or "version" not in project.get("dynamic", []):
    raise SystemExit("Python version must be dynamic from Cargo")
if args.tag is not None and args.tag != f"v{version}":
    raise SystemExit(f"Tag {args.tag!r} must be v{version}")
print(f"ForceLoom version verified: {version}")
