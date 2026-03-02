#!/usr/bin/env bash
# Generate pandoc reference PDFs and extracted text from test fixtures.
# Run once (or when fixtures change) to produce reference artifacts.
#
# Requires: pandoc, tectonic, pdftotext
# Usage: bash tests/generate_references.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FIXTURES="$SCRIPT_DIR/fixtures"
LEGACY="$SCRIPT_DIR/legacy"
PANDOC_OUT="$SCRIPT_DIR/references/pandoc"
TEXT_OUT="$SCRIPT_DIR/references/text"

mkdir -p "$PANDOC_OUT" "$TEXT_OUT"

# Check dependencies
for cmd in pandoc tectonic pdftotext; do
    if ! command -v "$cmd" &>/dev/null; then
        echo "error: $cmd not found on PATH" >&2
        exit 1
    fi
done

PREAMBLE="$LEGACY/preamble.tex"
if [[ ! -f "$PREAMBLE" ]]; then
    echo "error: preamble not found: $PREAMBLE" >&2
    exit 1
fi

for md in "$FIXTURES"/*.md; do
    name="$(basename "$md" .md)"
    pdf="$PANDOC_OUT/$name.pdf"
    txt="$TEXT_OUT/$name.txt"

    echo "  generating: $name"

    pandoc "$md" \
        -o "$pdf" \
        --pdf-engine=tectonic \
        -H "$PREAMBLE" \
        --toc \
        --number-sections \
        -V geometry:margin=1in \
        -V fontsize:11pt \
        2>/dev/null || {
            echo "  WARN: pandoc failed for $name, skipping"
            continue
        }

    pdftotext "$pdf" "$txt" 2>/dev/null || {
        echo "  WARN: pdftotext failed for $name"
    }
done

echo "done. Reference files in:"
echo "  PDFs: $PANDOC_OUT/"
echo "  Text: $TEXT_OUT/"
