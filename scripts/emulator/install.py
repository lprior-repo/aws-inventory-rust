#!/usr/bin/env python3
"""Fetch the pinned full source clone and verify/extract its official Linux binary.

No system install or service changes. Requires git, curl, gpg, dpkg-deb, Linux
x86_64. The source clone retains full upstream history. This script is opt-in.
"""

import argparse
import hashlib
import json
import platform
import subprocess
from pathlib import Path


def run(arguments):
    subprocess.run(arguments, check=True)


def fetch(url, path):
    run(["curl", "--fail", "--location", "--silent", "--show-error", "--max-time", "300", "--output", str(path), url])


def main(destination):
    metadata = json.loads((Path(__file__).parent / "upstream.json").read_text())
    if platform.system() != "Linux" or platform.machine() not in ("x86_64", "amd64"):
        raise SystemExit("This pinned package is Linux amd64; see upstream docs for other platforms.")
    destination = destination.resolve()
    destination.mkdir(parents=True, exist_ok=True)
    source = destination / "aws-emulator"
    if not source.exists():
        run(["git", "clone", metadata["repository"], str(source)])
        run(["git", "-C", str(source), "checkout", "--detach", metadata["commit"]])
    actual = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    shallow = subprocess.check_output(["git", "-C", str(source), "rev-parse", "--is-shallow-repository"], text=True).strip()
    if actual != metadata["commit"] or shallow != "false":
        raise SystemExit("Existing source does not match the pinned full clone; choose a new destination.")
    package = destination / "aws-emulator_amd64.deb"
    signature = destination / "aws-emulator_amd64.deb.sig"
    key = destination / "aws-publishing.asc"
    fetch(metadata["package_url"], package)
    if hashlib.file_digest(package.open("rb"), "sha256").hexdigest() != metadata["package_sha256"]:
        raise SystemExit("AWS Emulator package checksum mismatch")
    fetch(metadata["package_url"] + ".sig", signature)
    fetch(metadata["package_signing_key_url"], key)
    keyring = destination / "keyring"
    keyring.mkdir(mode=0o700, exist_ok=True)
    gpg = ["gpg", "--homedir", str(keyring), "--no-autostart", "--batch"]
    public_keys = keyring / "trusted.gpg"
    run(gpg + ["--yes", "--dearmor", "--output", str(public_keys), str(key)])
    fingerprints = subprocess.check_output(gpg + ["--with-colons", "--show-keys", str(public_keys)], text=True)
    if metadata["package_signing_key_fingerprint"] not in fingerprints:
        raise SystemExit("AWS package signing key fingerprint mismatch")
    verified = subprocess.run(gpg + ["--no-default-keyring", "--keyring", str(public_keys), "--status-fd", "1",
                                    "--verify", str(signature), str(package)],
                              check=True, capture_output=True)
    # Human-readable GPG diagnostics can contain non-UTF-8 key-user-ID bytes.
    # Only the machine-readable ASCII VALIDSIG status establishes the signer.
    valid_signatures = [line.decode("ascii").split() for line in verified.stdout.splitlines()
                        if line.startswith(b"[GNUPG:] VALIDSIG ")]
    expected_signer = metadata["package_signing_key_fingerprint"]
    if not any(parts[2] == expected_signer or parts[-1] == expected_signer for parts in valid_signatures):
        raise SystemExit("Package was not signed by the pinned AWS primary key")
    run(["dpkg-deb", "--extract", str(package), str(destination / "unpacked")])
    binary = destination / "unpacked/opt/aws-emulator/bin/aws-emulator"
    run([str(binary), "--version"])
    print(json.dumps({"source": str(source), "binary": str(binary), "verified_sha256": metadata["package_sha256"]}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    main(parser.parse_args().destination)
