import json
from pathlib import Path

import tomllib


def load_toml(path: Path) -> dict:
    return tomllib.loads(path.read_text())


root_path = Path("Cargo.toml")
root = load_toml(root_path)
members = root["workspace"]["members"]

if "." not in members or any("*" in member for member in members):
    raise SystemExit("workspace members must be explicit and include the root package")

manifests = {
    member: root_path if member == "." else Path(member, "Cargo.toml")
    for member in members
}
packages = {member: load_toml(path)["package"] for member, path in manifests.items()}
versions = {package["version"] for package in packages.values()}
msrv = root["workspace"]["package"]["rust-version"]

if len(versions) != 1:
    details = ", ".join(
        f"{package['name']}={package['version']}" for package in packages.values()
    )
    raise SystemExit(f"workspace package versions must match: {details}")
if any(
    package.get("rust-version") != {"workspace": True} for package in packages.values()
):
    raise SystemExit(f"every workspace package must inherit Rust {msrv}")

version = versions.pop()
manifest = json.loads(Path(".release-please-manifest.json").read_text())
config = json.loads(Path("release-please-config.json").read_text())

if manifest != {".": version}:
    raise SystemExit("release manifest must track the uniform root package version")
if set(config["packages"]) != {"."}:
    raise SystemExit(
        "Release Please must use the root package as the only release unit"
    )

for member, path in manifests.items():
    dependencies = load_toml(path).get("dependencies", {})
    for dependency, specification in dependencies.items():
        if not isinstance(specification, dict) or "path" not in specification:
            continue
        dependency_path = (path.parent / specification["path"] / "Cargo.toml").resolve()
        dependency_version = load_toml(dependency_path)["package"]["version"]
        if specification.get("version") != dependency_version:
            raise SystemExit(
                f"{member}: {dependency} must require workspace version {dependency_version}"
            )

print(f"release state is coordinated at {version}")
