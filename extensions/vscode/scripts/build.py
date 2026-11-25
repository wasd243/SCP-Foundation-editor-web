import argparse
import json
import platform
import shutil
import subprocess
from pathlib import Path


EXTENSION_ROOT = Path(__file__).resolve().parents[1]
PLATFORMS = {
    "win32-x64": ("ftml_analyzer.exe", "win32-x64"),
    "win32-arm64": ("ftml_analyzer.exe", "win32-arm64"),
    "linux-x64": ("ftml_analyzer", "linux-x64"),
    "linux-arm64": ("ftml_analyzer", "linux-arm64"),
    "darwin-x64": ("ftml_analyzer", "darwin-x64"),
    "darwin-arm64": ("ftml_analyzer", "darwin-arm64"),
}


def host_target() -> str:
    operating_system = {"Windows": "win32", "Linux": "linux", "Darwin": "darwin"}.get(platform.system())
    architecture = {
        "AMD64": "x64",
        "x86_64": "x64",
        "aarch64": "arm64",
        "ARM64": "arm64",
    }.get(platform.machine())
    if operating_system is None or architecture is None:
        raise ValueError(f"Unsupported build host: {platform.system()} {platform.machine()}")
    return f"{operating_system}-{architecture}"


def stage(target: str, source: Path | None):
    if target not in PLATFORMS:
        raise ValueError(f"Unsupported extension target: {target}")

    binary, _ = PLATFORMS[target]
    source = source or EXTENSION_ROOT.parents[1] / "target" / "release" / binary
    source = source.resolve()
    if not source.is_file():
        raise FileNotFoundError(f"Language server binary not found: {source}")

    destination = EXTENSION_ROOT / "bin" / target / binary
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    if not target.startswith("win32-"):
        destination.chmod(0o755)
    print(f"Staged {source} -> {destination}")


def check_binaries():
    for target, (binary, _) in PLATFORMS.items():
        executable = EXTENSION_ROOT / "bin" / target / binary
        if not executable.is_file():
            raise FileNotFoundError(f"Missing {target} language server: {executable}")
        if not target.startswith("win32-"):
            executable.chmod(0o755)
    print(f"Found and prepared {len(PLATFORMS)} platform LSP binaries.")


def package_local():
    target = host_target()
    if target not in PLATFORMS:
        raise ValueError(f"Cannot package an extension for unsupported host: {target}")

    node = shutil.which("node")
    if not node:
        raise RuntimeError("Node.js is required to run VSCE.")
    vsce = EXTENSION_ROOT / "node_modules" / "@vscode" / "vsce" / "vsce"
    subprocess.run(
        [node, str(vsce), "package", "--no-dependencies", "--target", PLATFORMS[target][1]],
        cwd=EXTENSION_ROOT,
        check=True,
    )


def check_version(tag: str):
    manifest = json.loads((EXTENSION_ROOT / "package.json").read_text(encoding="utf-8"))
    version = manifest["version"]
    tag_version = tag.removeprefix("v")
    if tag_version != version:
        raise ValueError(f"Tag {tag} does not match extension version {version}")
    print(f"Release tag {tag} matches extension version {version}.")


def main():
    parser = argparse.ArgumentParser(description="Build and package the FTML VS Code extension.")
    commands = parser.add_subparsers(dest="command", required=True)

    stage_command = commands.add_parser("stage", help="Copy a built language server into the extension.")
    stage_command.add_argument("--target", default=host_target())
    stage_command.add_argument("--source", type=Path)

    commands.add_parser("check-binaries", help="Validate all platform server binaries.")

    version_command = commands.add_parser("check-version", help="Validate a release tag against package.json.")
    version_command.add_argument("tag")

    commands.add_parser("package-local", help="Create a VSIX for the current host platform.")

    args = parser.parse_args()
    if args.command == "stage":
        stage(args.target, args.source)
    elif args.command == "check-binaries":
        check_binaries()
    elif args.command == "check-version":
        check_version(args.tag)
    elif args.command == "package-local":
        package_local()


if __name__ == "__main__":
    main()
