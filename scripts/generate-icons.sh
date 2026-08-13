#!/bin/bash
# Generate app icons from a source 1024x1024 PNG
# Usage: ./scripts/generate-icons.sh [source.png]
#
# If no source is provided, defaults to src-tauri/icons/app-icon-1024.png

set -euo pipefail

SOURCE="${1:-src-tauri/icons/app-icon-1024.png}"
OUTPUT_DIR="src-tauri/icons"

if [ ! -f "$SOURCE" ]; then
  echo "Error: Source file '$SOURCE' not found."
  echo "Please provide a 1024x1024 PNG as the first argument."
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

if command -v sips &> /dev/null; then
  echo "Using sips (macOS)..."

  sips -z 32 32 "$SOURCE" --out "$OUTPUT_DIR/32x32.png"
  sips -z 128 128 "$SOURCE" --out "$OUTPUT_DIR/128x128.png"
  sips -z 256 256 "$SOURCE" --out "$OUTPUT_DIR/128x128@2x.png"
  sips -z 256 256 "$SOURCE" --out "$OUTPUT_DIR/256x256.png"

  # Generate .icns using iconutil
  ICONSET_DIR=$(mktemp -d)/AppIcon.iconset
  mkdir -p "$ICONSET_DIR"
  sips -z 16 16 "$SOURCE" --out "$ICONSET_DIR/icon_16x16.png"
  sips -z 32 32 "$SOURCE" --out "$ICONSET_DIR/icon_16x16@2x.png"
  sips -z 32 32 "$SOURCE" --out "$ICONSET_DIR/icon_32x32.png"
  sips -z 64 64 "$SOURCE" --out "$ICONSET_DIR/icon_32x32@2x.png"
  sips -z 128 128 "$SOURCE" --out "$ICONSET_DIR/icon_128x128.png"
  sips -z 256 256 "$SOURCE" --out "$ICONSET_DIR/icon_128x128@2x.png"
  sips -z 256 256 "$SOURCE" --out "$ICONSET_DIR/icon_256x256.png"
  sips -z 512 512 "$SOURCE" --out "$ICONSET_DIR/icon_256x256@2x.png"
  sips -z 512 512 "$SOURCE" --out "$ICONSET_DIR/icon_512x512.png"
  sips -z 1024 1024 "$SOURCE" --out "$ICONSET_DIR/icon_512x512@2x.png"
  iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_DIR/icon.icns"
  rm -rf "$(dirname "$ICONSET_DIR")"

  echo "Generated .icns"

elif command -v magick &> /dev/null; then
  echo "Using ImageMagick..."

  magick "$SOURCE" -resize 32x32 "$OUTPUT_DIR/32x32.png"
  magick "$SOURCE" -resize 128x128 "$OUTPUT_DIR/128x128.png"
  magick "$SOURCE" -resize 256x256 "$OUTPUT_DIR/128x128@2x.png"
  magick "$SOURCE" -resize 256x256 "$OUTPUT_DIR/256x256.png"

  # Generate .ico (multi-size)
  magick "$SOURCE" \
    -define icon:auto-resize=256,128,64,48,32,16 \
    "$OUTPUT_DIR/icon.ico"

  echo "Generated .ico"
  echo "Note: .icns generation requires macOS iconutil. Use sips on macOS."

else
  echo "Error: Neither 'sips' (macOS) nor 'magick' (ImageMagick) found."
  echo "Install ImageMagick: brew install imagemagick"
  exit 1
fi

echo ""
echo "Icons generated in $OUTPUT_DIR:"
ls -la "$OUTPUT_DIR"
