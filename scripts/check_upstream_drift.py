#!/usr/bin/env python3
"""Report drift between this crate and the newest `google-antigravity` release.

This crate fell three minor versions behind upstream — including a wire-breaking
rename — because nothing was watching. Two checks run here:

1. **Version.** Is there a release newer than the one `proto/localharness.proto`
   was generated from?
2. **Schema.** Does the proto regenerated from that release differ from the one
   in the repository?

Check 2 is the one that matters: a version bump with no schema change is a
five-minute pin update, whereas a schema change is a migration. Both are
reported; the exit status is non-zero if either fires.

Usage:
    pip install protobuf requests
    python3 scripts/check_upstream_drift.py [--pypi-url URL]

Run from the repository root. Offline or rate-limited runs report `SKIP` and
exit 0 — a flaky network must not read as a drift.
"""

from __future__ import annotations

import argparse
import io
import json
import pathlib
import re
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
import zipfile

PACKAGE = "google-antigravity"
PYPI_JSON = "https://pypi.org/pypi/{package}/json"
REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
PROTO_PATH = REPO_ROOT / "proto" / "localharness.proto"
INSTALL_SCRIPT = REPO_ROOT / "scripts" / "install_harness.sh"
GEN_PROTO = REPO_ROOT / "scripts" / "gen_proto.py"

# The version the checked-in proto was generated from. Update this in the same
# commit that regenerates the proto.
PINNED_VERSION = "0.1.9"


def check_install_script_pin() -> bool:
    """True when install_harness.sh downloads the version the proto came from."""
    try:
        text = INSTALL_SCRIPT.read_text()
    except OSError as exc:
        _skip(f"could not read {INSTALL_SCRIPT.name}: {exc}")
        return True
    match = re.search(r'^VERSION="([^"]+)"', text, re.MULTILINE)
    if match is None:
        _fail(f"{INSTALL_SCRIPT.name} has no VERSION= line to check")
        return False
    if match.group(1) != PINNED_VERSION:
        _fail(
            f"{INSTALL_SCRIPT.name} installs harness {match.group(1)}, but the "
            f"proto was generated from {PINNED_VERSION}. The installed harness "
            f"and the wire format this SDK speaks must be the same version."
        )
        return False
    print(f"OK: {INSTALL_SCRIPT.name} installs the pinned {PINNED_VERSION}")
    return True


def _fail(message: str) -> None:
    print(f"DRIFT: {message}")


def _skip(message: str) -> None:
    print(f"SKIP: {message}")


def fetch_release_metadata(pypi_url: str) -> dict | None:
    """Returns the PyPI metadata, or None when it cannot be reached."""
    try:
        with urllib.request.urlopen(pypi_url, timeout=30) as response:
            return json.loads(response.read())
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as error:
        _skip(f"could not reach PyPI ({error})")
        return None


def parse_version(version: str) -> tuple[int, ...]:
    """`0.1.10` sorts after `0.1.9`, which a string comparison gets wrong."""
    parts = []
    for piece in version.split("."):
        digits = "".join(c for c in piece if c.isdigit())
        parts.append(int(digits) if digits else 0)
    return tuple(parts)


def newest_version(metadata: dict) -> str | None:
    releases = [v for v, files in metadata.get("releases", {}).items() if files]
    if not releases:
        return None
    return max(releases, key=parse_version)


def wheel_url(metadata: dict, version: str) -> str | None:
    for artifact in metadata.get("releases", {}).get(version, []):
        if artifact.get("packagetype") == "bdist_wheel":
            return artifact.get("url")
    return None


def regenerate_proto(url: str) -> str | None:
    """Downloads the wheel and renders the `.proto` it embeds."""
    try:
        with urllib.request.urlopen(url, timeout=120) as response:
            payload = response.read()
    except (urllib.error.URLError, TimeoutError) as error:
        _skip(f"could not download the wheel ({error})")
        return None

    with tempfile.TemporaryDirectory() as workdir:
        try:
            with zipfile.ZipFile(io.BytesIO(payload)) as wheel:
                wheel.extractall(workdir)
        except zipfile.BadZipFile as error:
            _skip(f"the wheel is not readable ({error})")
            return None

        result = subprocess.run(
            [sys.executable, str(GEN_PROTO), workdir],
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            _skip(f"gen_proto.py failed: {result.stderr.strip()[:400]}")
            return None
        return result.stdout


def schema_body(proto_text: str) -> str:
    """The schema without its provenance header.

    `gen_proto.py` stamps the directory it read from into a `// Source:` line,
    which differs between a checkout and a freshly unpacked wheel. Comparing it
    would report drift on every run and train everyone to ignore this job.
    """
    lines = [
        line.rstrip()
        for line in proto_text.splitlines()
        if not line.startswith("// Source:")
    ]
    return "\n".join(lines).strip()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--pypi-url",
        default=PYPI_JSON.format(package=PACKAGE),
        help="Override the metadata URL (for testing).",
    )
    args = parser.parse_args()

    metadata = fetch_release_metadata(args.pypi_url)
    if metadata is None:
        return 0

    latest = newest_version(metadata)
    if latest is None:
        _skip("PyPI reported no releases")
        return 0

    drifted = False

    # The install script downloads the harness developers actually run. It
    # pinned 0.1.1 for the whole 0.1.9 migration -- so it handed out a harness
    # this SDK can no longer complete a turn against, and nothing noticed
    # because the two pins were never compared.
    if not check_install_script_pin():
        drifted = True

    if parse_version(latest) > parse_version(PINNED_VERSION):
        _fail(
            f"{PACKAGE} {latest} is newer than the pinned {PINNED_VERSION}. "
            f"Regenerate proto/localharness.proto and update PINNED_VERSION in "
            f"this script."
        )
        drifted = True
    else:
        print(f"OK: pinned {PINNED_VERSION} is the newest release")

    url = wheel_url(metadata, latest)
    if url is None:
        _skip(f"{latest} publishes no wheel; the schema check needs one")
        return 1 if drifted else 0

    regenerated = regenerate_proto(url)
    if regenerated is None:
        return 1 if drifted else 0

    current = PROTO_PATH.read_text()
    if schema_body(regenerated) != schema_body(current):
        _fail(
            f"proto/localharness.proto differs from what {PACKAGE} {latest} ships. "
            f"This is the check that matters: a schema change is a migration, not "
            f"a pin bump. Regenerate with:\n"
            f"    python3 scripts/gen_proto.py <unpacked-wheel> > proto/localharness.proto"
        )
        drifted = True
    else:
        print(f"OK: the checked-in proto matches {PACKAGE} {latest}")

    return 1 if drifted else 0


if __name__ == "__main__":
    sys.exit(main())
