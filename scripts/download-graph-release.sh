#!/bin/bash
# Script to download and extract the latest iqrah-graph release
# Usage: ./scripts/download-graph-release.sh [version] [output_dir]

set -e

VERSION=${1:-"v1.1.0"}
OUTPUT_DIR=${2:-"assets"}
RELEASE_TAG="iqrah-graph-${VERSION}"
RELEASE_URL="https://github.com/iqrahapp/iqrah-mobile/releases/download/${RELEASE_TAG}/iqrah-graph-${VERSION}.tar.gz"

echo "📥 Downloading iqrah-graph ${VERSION}..."
echo "   Release URL: ${RELEASE_URL}"
echo "   Output directory: ${OUTPUT_DIR}"

# Create output directory if it doesn't exist
mkdir -p "${OUTPUT_DIR}"

# Download the release
if command -v wget &> /dev/null; then
    wget -q --show-progress "${RELEASE_URL}" -O /tmp/iqrah-graph.tar.gz
elif command -v curl &> /dev/null; then
    curl -L "${RELEASE_URL}" -o /tmp/iqrah-graph.tar.gz
else
    echo "❌ Error: Neither wget nor curl found. Please install one of them."
    exit 1
fi

# Extract the graph file
echo "📦 Extracting knowledge graph..."
tar -xzf /tmp/iqrah-graph.tar.gz -C "${OUTPUT_DIR}/"

if [ -f "${OUTPUT_DIR}/knowledge-graph.cbor.zst" ]; then
    echo "✅ Extracted: knowledge-graph.cbor.zst"
else
    echo "❌ Error: knowledge-graph.cbor.zst not found in tarball"
    ls -la "${OUTPUT_DIR}/"
    exit 1
fi

# Clean up
rm /tmp/iqrah-graph.tar.gz

# Verify the CBOR file is valid zstandard compressed
echo ""
echo "🔍 Verifying downloaded file..."
if command -v zstd &> /dev/null; then
    if zstd -t "${OUTPUT_DIR}/knowledge-graph.cbor.zst" &> /dev/null; then
        echo "✅ CBOR file is valid zstandard compressed"
    else
        echo "❌ Error: CBOR file failed zstandard validation"
        exit 1
    fi
else
    echo "⚠️  zstd not installed, skipping validation"
fi

# Show file info
echo ""
echo "📊 Downloaded file:"
ls -lh "${OUTPUT_DIR}/knowledge-graph.cbor.zst"

echo ""
echo "✅ iqrah-graph ${VERSION} ready in ${OUTPUT_DIR}/"
