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
tar -xzf /tmp/iqrah-graph.tar.gz -C /tmp/

# Move to output directory
mv /tmp/knowledge-graph.cbor.zst "${OUTPUT_DIR}/"

# Also extract content.db if it exists in the tarball
if tar -tzf /tmp/iqrah-graph.tar.gz | grep -q "content.db"; then
    tar -xzf /tmp/iqrah-graph.tar.gz -C /tmp/ content.db
    mv /tmp/content.db "${OUTPUT_DIR}/"
    echo "✅ Downloaded: knowledge-graph.cbor.zst and content.db"
else
    echo "✅ Downloaded: knowledge-graph.cbor.zst"
fi

# Clean up
rm /tmp/iqrah-graph.tar.gz

# Show file info
echo ""
echo "📊 Downloaded files:"
ls -lh "${OUTPUT_DIR}"/knowledge-graph.cbor.zst
if [ -f "${OUTPUT_DIR}/content.db" ]; then
    ls -lh "${OUTPUT_DIR}"/content.db
fi

echo ""
echo "✅ iqrah-graph ${VERSION} ready in ${OUTPUT_DIR}/"
