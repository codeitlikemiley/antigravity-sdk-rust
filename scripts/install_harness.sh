#!/bin/bash
# install_harness.sh - Download and extract the compiled localharness binary
# from the PyPI wheel without requiring Python, pip, or virtual environments.

set -euo pipefail

VERSION="0.1.1"
PLATFORM=""
case "$(uname -s)" in
    Darwin)
        case "$(uname -m)" in
            arm64) PLATFORM="macosx_11_0_arm64" ;;
            x86_64) PLATFORM="macosx_10_9_x86_64" ;;
            *) echo "Unsupported macOS architecture"; exit 1 ;;
        esac
        ;;
    Linux)
        case "$(uname -m)" in
            x86_64) PLATFORM="manylinux_2_17_x86_64" ;;
            aarch64) PLATFORM="manylinux_2_17_aarch64" ;;
            *) echo "Unsupported Linux architecture"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported operating system"
        exit 1
        ;;
esac

echo "Fetching localharness v${VERSION} for ${PLATFORM}..."

# Query the PyPI JSON API to locate the download URL for the wheel matching our platform
JSON_DATA=$(curl -sSf "https://pypi.org/pypi/google-antigravity/${VERSION}/json" || true)
DOWNLOAD_URL=""
if [ -n "$JSON_DATA" ]; then
    DOWNLOAD_URL=$(echo "$JSON_DATA" | grep -o 'https://[^"]*\.whl' | grep "$PLATFORM" | head -n 1 || true)
fi

if [ -z "$DOWNLOAD_URL" ]; then
    # Fallback to standard URL construction if grep parsing fails
    DOWNLOAD_URL="https://files.pythonhosted.org/packages/py3/g/google-antigravity/google_antigravity-${VERSION}-py3-none-${PLATFORM}.whl"
fi

WHEEL_FILE="google_antigravity-${VERSION}-py3-none-${PLATFORM}.whl"

echo "Downloading wheel from: ${DOWNLOAD_URL}"
if ! curl -sSLf -o "${WHEEL_FILE}" "${DOWNLOAD_URL}"; then
    echo "ERROR: Failed to download the localharness wheel for platform '${PLATFORM}'."
    echo "This platform version may not be supported or published on PyPI."
    exit 1
fi

echo "Extracting localharness binary..."
# Extract just the binary from the wheel (which is a standard ZIP file)
unzip -q -o "${WHEEL_FILE}" "google/antigravity/bin/localharness"

# Move the binary to the bin/ directory
mkdir -p bin
mv google/antigravity/bin/localharness bin/localharness
chmod +x bin/localharness

# Clean up temporary files
rm -rf google "${WHEEL_FILE}"

echo "SUCCESS: localharness installed at ./bin/localharness"
echo "To configure, run: export ANTIGRAVITY_HARNESS_PATH=\$(pwd)/bin/localharness"
