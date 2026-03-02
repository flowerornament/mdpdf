#!/usr/bin/env bash
# Visual comparison: render fixtures with both typst (mdpdf) and pandoc,
# convert to PNGs, optionally generate pixel diffs.
#
# Requires: mdpdf (cargo build), pandoc, tectonic, pdftoppm
# Optional: magick (ImageMagick) for diff images
#
# Usage: bash tests/visual.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
FIXTURES="$SCRIPT_DIR/fixtures"
LEGACY="$SCRIPT_DIR/legacy"
OUTPUT="$SCRIPT_DIR/output/visual"

mkdir -p "$OUTPUT/typst" "$OUTPUT/pandoc" "$OUTPUT/diff"

PREAMBLE="$LEGACY/preamble.tex"
MDPDF="$PROJECT_DIR/target/release/mdpdf"

if [[ ! -x "$MDPDF" ]]; then
    MDPDF="$PROJECT_DIR/target/debug/mdpdf"
fi

if [[ ! -x "$MDPDF" ]]; then
    echo "Building mdpdf..."
    (cd "$PROJECT_DIR" && cargo build --release)
    MDPDF="$PROJECT_DIR/target/release/mdpdf"
fi

HAS_MAGICK=false
if command -v magick &>/dev/null; then
    HAS_MAGICK=true
fi

for md in "$FIXTURES"/*.md; do
    name="$(basename "$md" .md)"
    echo "=== $name ==="

    # Typst render
    typst_pdf="$OUTPUT/typst/$name.pdf"
    "$MDPDF" "$md" -o "$typst_pdf" --no-toc --no-number-sections 2>/dev/null || {
        echo "  WARN: mdpdf failed for $name"
        continue
    }
    pdftoppm -png "$typst_pdf" "$OUTPUT/typst/$name" 2>/dev/null

    # Pandoc render
    pandoc_pdf="$OUTPUT/pandoc/$name.pdf"
    if command -v pandoc &>/dev/null && command -v tectonic &>/dev/null; then
        pandoc "$md" \
            -o "$pandoc_pdf" \
            --pdf-engine=tectonic \
            -H "$PREAMBLE" \
            -V geometry:margin=1in \
            -V fontsize:11pt \
            2>/dev/null || {
                echo "  WARN: pandoc failed for $name"
                continue
            }
        pdftoppm -png "$pandoc_pdf" "$OUTPUT/pandoc/$name" 2>/dev/null

        # Pixel diff (if ImageMagick available)
        if $HAS_MAGICK; then
            for typst_png in "$OUTPUT/typst/$name"-*.png; do
                page="$(basename "$typst_png" .png)"
                pandoc_png="$OUTPUT/pandoc/$page.png"
                if [[ -f "$pandoc_png" ]]; then
                    magick compare "$typst_png" "$pandoc_png" \
                        "$OUTPUT/diff/$page.png" 2>/dev/null || true
                fi
            done
        fi
    else
        echo "  SKIP pandoc (not installed)"
    fi
done

echo ""
echo "Output in: $OUTPUT/"
echo "  typst/   — typst-rendered PNGs"
echo "  pandoc/  — pandoc-rendered PNGs"
echo "  diff/    — pixel diff images (if ImageMagick available)"
