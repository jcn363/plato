#! /bin/sh
# Converts a StarDict dictionary to the dictd format.
# The first argument must be the path to the IFO file.

trap 'exit 1' ERR

base=${1%.*}
bindir=bin/utils
short_name=$(grep '^bookname=' "$1" | cut -d '=' -f 2)
url=$(grep '^website=' "$1" | cut -d '=' -f 2)

echo "Converting ${short_name} (${1})..."

# Count total lines for progress
if [ -f "${base}.idx" ]; then
    total_lines=$(wc -l < "${base}.idx")
    echo "Processing ${total_lines} entries..."
fi

# Step 1: Decompress if needed
echo "[1/5] Decompressing dict.dz..."
[ -e "${base}.dict.dz" ] && "$bindir"/dictzip -d "${base}.dict.dz"

# Step 2: Convert idx to txt
echo "[2/5] Converting index file..."
args="${base}.dict"
[ -e "${base}.syn" ] && args="$args ${base}.syn"

# shellcheck disable=SC2086
"$bindir"/sdunpack $args < "${base}.idx" > "${base}.txt"

# Step 3: Post-process Wiktionary
if [ "${short_name%% *}" = "Wiktionary" ]; then
    echo "[3/5] Post-processing Wiktionary entries..."
    sed -i 's/^\([\[/].*\)/<p>\1<\/p>/' "${base}.txt"
else
    echo "[3/5] Post-processing entries..."
fi

# Step 4: Build dict format
echo "[4/5] Building dictionary format..."
"$bindir"/dictfmt --quiet --utf8 --index-keep-orig --headword-separator '|' -s "$short_name" -u "$url" -t "$base" < "${base}.txt"

# Step 5: Compress dict
echo "[5/5] Compressing dictionary..."
"$bindir"/dictzip "${base}.dict"

# Cleanup
echo "Cleaning up temporary files..."
rm "$1" "${base}.idx" "${base}.txt"
[ -e "${base}.syn" ] && rm "${base}.syn"

echo "Done! Dictionary converted successfully."
echo ""
echo "Files created:"
echo "  - ${base}.dict.dz"
echo "  - ${base}.index"
