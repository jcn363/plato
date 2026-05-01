#!/usr/bin/env bash
set -euo pipefail

# 1. Build list of symbols
SYMBOLS_FILE=$(mktemp)
(
  # public items
  grep -rEho 'pub (fn|struct|enum|trait|const|static|type|mod) ([A-Za-z_][A-Za-z0-9_]*)' crates |
    awk '{print $3}'

  # crate names
  git ls-files crates/*/Cargo.toml | while read -r cargo; do
    awk '/^\s*name\s*=/{gsub(/[\" ]/, "", $2); print $2}' "$cargo"
  done

  # const/static inside modules
  grep -rEho '(const|static) ([A-Z_][A-Z0-9_]*)' crates |
    awk '{print $2}'
) | sort -u >"$SYMBOLS_FILE"

# 2. Scan docs
MISMATCHES=()
DOCS=$(find doc -name '*.md' -print0)

while IFS= read -r -d '' doc; do
  tokens=$(grep -oP '`[^`]*`' "$doc" | tr -d '`')
  for tok in $tokens; do
    if [[ "$tok" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
      case "$tok" in
        fn|struct|enum|trait|impl|mod|pub|crate|self|self::|super|super::) continue ;;
      esac
      if ! grep -qFx "$tok" "$SYMBOLS_FILE"; then
        MISMATCHES+=("$doc: \`$tok\`")
      fi
    fi
  done
done < <(printf '%s\0' $DOCS)

if [[ ${#MISMATCHES[@]} -eq 0 ]]; then
  echo "✅  No stale links or identifiers found – the docs match the source."
else
  echo "⚠️  Potentially stale or nonexistent identifiers were found:"
  printf '%s\n' "${MISMATCHES[@]}"
  echo
  echo "Next steps:"
  echo "  1) Open each file listed above."
  echo "  2) Verify whether the identifier should really be there."
  echo "  3) If the symbol was renamed or removed, update the markdown."
  echo "  4) If the symbol still exists but was missed by the parser (e.g. due to complex syntax), ignore or add it to the symbol list."
fi

rm -f "$SYMBOLS_FILE"
