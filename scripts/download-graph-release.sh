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

# Clean up any old graph files in output directory
echo "🧹 Cleaning up old graph files..."
rm -f "${OUTPUT_DIR}"/knowledge-graph.cbor.zst
rm -f "${OUTPUT_DIR}"/iqrah-graph-*.cbor.zst

# Extract the graph file
echo "📦 Extracting knowledge graph..."
# Extract to a temporary directory to avoid conflicts
EXTRACT_DIR=$(mktemp -d)
tar -xzf /tmp/iqrah-graph.tar.gz -C "${EXTRACT_DIR}/"

# Move to output directory
if [ -f "${EXTRACT_DIR}/knowledge-graph.cbor.zst" ]; then
    mv "${EXTRACT_DIR}/knowledge-graph.cbor.zst" "${OUTPUT_DIR}/"
    echo "✅ Extracted: knowledge-graph.cbor.zst"
else
    echo "❌ Error: knowledge-graph.cbor.zst not found in tarball"
    ls -la "${EXTRACT_DIR}/"
    exit 1
fi

# Also extract content.db if it exists in the tarball
if [ -f "${EXTRACT_DIR}/content.db" ]; then
    mv "${EXTRACT_DIR}/content.db" "${OUTPUT_DIR}/"
    echo "✅ Extracted: content.db"
fi

# Clean up extraction directory
rm -rf "${EXTRACT_DIR}"

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
echo "📊 Downloaded files:"
ls -lh "${OUTPUT_DIR}"/knowledge-graph.cbor.zst
if [ -f "${OUTPUT_DIR}/content.db" ]; then
    ls -lh "${OUTPUT_DIR}"/content.db
fi

echo ""
echo "✅ iqrah-graph ${VERSION} ready in ${OUTPUT_DIR}/"
