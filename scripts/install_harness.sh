#!/bin/bash
# install_harness.sh - Download and extract the compiled localharness binary
# from the PyPI wheel without requiring Python, pip, or virtual environments.

set -euo pipefail

# Must match PINNED_VERSION in scripts/check_upstream_drift.py, which enforces
# it. The SDK speaks this version's wire format: against 0.1.1 a turn never
# ends, because STATE_IDLE was renamed STATE_FULLY_IDLE in 0.1.9 and protojson
# drops the unknown variant.
VERSION="0.1.9"
PLATFORM=""
ARCH="$(uname -m)"
OS="$(uname -s)"

# Normalize OS
case "$OS" in
    Darwin) OS_MATCH="macosx" ;;
    Linux) OS_MATCH="linux" ;;
    MINGW*|MSYS*|CYGWIN*) OS_MATCH="win" ;;
    *) OS_MATCH="unknown" ;;
esac

# Normalize ARCH
case "$ARCH" in
    x86_64|amd64|AMD64)
        if [ "$OS_MATCH" = "win" ]; then
            ARCH_MATCH="amd64"
        else
            ARCH_MATCH="x86_64"
        fi
        ;;
    arm64|aarch64|ARM64)
        if [ "$OS_MATCH" = "linux" ]; then
            ARCH_MATCH="aarch64"
        else
            ARCH_MATCH="arm64"
        fi
        ;;
    *)
        ARCH_MATCH="unknown"
        ;;
esac

# Default / Guess platform strings for fallback if PyPI JSON API is unreachable
FALLBACK_PLATFORM=""
if [ "$OS_MATCH" = "macosx" ]; then
    if [ "$ARCH_MATCH" = "arm64" ]; then
        FALLBACK_PLATFORM="macosx_11_0_arm64"
    else
        FALLBACK_PLATFORM="macosx_10_9_x86_64"
    fi
elif [ "$OS_MATCH" = "linux" ]; then
    if [ "$ARCH_MATCH" = "aarch64" ]; then
        FALLBACK_PLATFORM="manylinux_2_17_aarch64"
    else
        FALLBACK_PLATFORM="manylinux_2_17_x86_64"
    fi
elif [ "$OS_MATCH" = "win" ]; then
    if [ "$ARCH_MATCH" = "arm64" ]; then
        FALLBACK_PLATFORM="win_arm64"
    else
        FALLBACK_PLATFORM="win_amd64"
    fi
fi

echo "Detecting localharness v${VERSION} for OS='${OS}' (${OS_MATCH}), ARCH='${ARCH}' (${ARCH_MATCH})..."

# Query the PyPI JSON API to locate the download URL for the wheel matching our platform
JSON_DATA=$(curl -sSf "https://pypi.org/pypi/google-antigravity/${VERSION}/json" || true)
DOWNLOAD_URL=""
if [ -n "$JSON_DATA" ] && [ "$OS_MATCH" != "unknown" ] && [ "$ARCH_MATCH" != "unknown" ]; then
    # Extract all .whl URLs and filter for ones matching both OS_MATCH and ARCH_MATCH
    WHEEL_URLS=$(echo "$JSON_DATA" | grep -o 'https://[^"]*\.whl' || true)
    if [ -n "$WHEEL_URLS" ]; then
        DOWNLOAD_URL=$(echo "$WHEEL_URLS" | grep "$OS_MATCH" | grep "$ARCH_MATCH" | head -n 1 || true)
    fi
fi

if [ -z "$DOWNLOAD_URL" ] && [ -n "$FALLBACK_PLATFORM" ]; then
    # Fallback to standard URL construction if grep/JSON parsing or API connection fails
    echo "PyPI API lookup skipped or failed; using constructed fallback URL..."
    DOWNLOAD_URL="https://files.pythonhosted.org/packages/py3/g/google-antigravity/google_antigravity-${VERSION}-py3-none-${FALLBACK_PLATFORM}.whl"
fi

if [ -z "$DOWNLOAD_URL" ]; then
    echo "============================================================"
    echo "ERROR: Operating system '${OS}' or architecture '${ARCH}' is not supported."
    echo "============================================================"
    echo ""
    echo "The upstream google-antigravity package on PyPI does not"
    echo "provide a valid wheel for this platform."
    echo "See: https://pypi.org/project/google-antigravity/#files"
    echo ""
    echo "Recommended alternatives:"
    echo "  1. Use Apple Silicon (arm64) macOS or Linux (x86_64/aarch64)"
    echo "  2. Run inside a Linux container or VM (Docker, OrbStack, etc.)"
    echo ""
    exit 1
fi

WHEEL_FILE=$(basename "$DOWNLOAD_URL")

echo "Downloading wheel from: ${DOWNLOAD_URL}"
if ! curl -sSLf -o "${WHEEL_FILE}" "${DOWNLOAD_URL}"; then
    echo "============================================================"
    echo "ERROR: Failed to download the localharness wheel."
    echo "============================================================"
    echo "URL: ${DOWNLOAD_URL}"
    echo "This platform may not be supported or published on PyPI."
    echo "Please check: https://pypi.org/project/google-antigravity/#files"
    exit 1
fi

# Validate the downloaded wheel is not empty or corrupt
if [[ ! -s "${WHEEL_FILE}" ]]; then
    echo "============================================================"
    echo "ERROR: Downloaded wheel is empty (0 bytes)."
    echo "============================================================"
    echo "This usually means the platform is not supported upstream."
    echo "Check: https://pypi.org/project/google-antigravity/#files"
    rm -f "${WHEEL_FILE}"
    exit 1
fi

# Validate it is a valid ZIP (wheel) file
if ! unzip -t "${WHEEL_FILE}" > /dev/null 2>&1; then
    echo "============================================================"
    echo "ERROR: Downloaded wheel is not a valid ZIP/wheel file."
    echo "============================================================"
    echo "The file may be corrupt or the platform may be unsupported upstream."
    rm -f "${WHEEL_FILE}"
    exit 1
fi

# Locate the binary inside the wheel dynamically
BINARY_PATH_IN_WHEEL=$(unzip -l "${WHEEL_FILE}" | grep -o 'google/antigravity/bin/localharness\(\.exe\)\?' | head -n 1 || true)

if [ -z "$BINARY_PATH_IN_WHEEL" ]; then
    echo "============================================================"
    echo "ERROR: Could not find 'localharness' binary in the downloaded wheel."
    echo "============================================================"
    rm -f "${WHEEL_FILE}"
    exit 1
fi

echo "Extracting localharness binary..."
if ! unzip -q -o "${WHEEL_FILE}" "${BINARY_PATH_IN_WHEEL}"; then
    echo "ERROR: Could not extract '${BINARY_PATH_IN_WHEEL}' from wheel."
    rm -f "${WHEEL_FILE}"
    exit 1
fi

BINARY_FILENAME=$(basename "$BINARY_PATH_IN_WHEEL")

# Move the binary to the bin/ directory
mkdir -p bin
mv "${BINARY_PATH_IN_WHEEL}" "bin/${BINARY_FILENAME}"
chmod +x "bin/${BINARY_FILENAME}"

# Validate the extracted binary is not empty
if [[ ! -s "bin/${BINARY_FILENAME}" ]]; then
    echo "ERROR: Extracted localharness binary is empty (0 bytes)."
    rm -f "bin/${BINARY_FILENAME}"
    rm -rf google "${WHEEL_FILE}"
    exit 1
fi

# Clean up temporary files
rm -rf google "${WHEEL_FILE}"

echo "SUCCESS: localharness installed at ./bin/${BINARY_FILENAME}"
echo "To configure, run: export ANTIGRAVITY_HARNESS_PATH=\$(pwd)/bin/${BINARY_FILENAME}"
