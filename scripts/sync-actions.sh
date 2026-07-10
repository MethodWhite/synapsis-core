#!/bin/bash
# Reads .github/versions.yml and applies versions to all workflow files
# Usage: ./scripts/sync-actions.sh

VERSIONS_FILE=".github/versions.yml"
WORKFLOWS_DIR=".github/workflows"

if [ ! -f "$VERSIONS_FILE" ]; then
  echo "Error: $VERSIONS_FILE not found"
  exit 1
fi

echo "Syncing action versions from $VERSIONS_FILE..."
echo ""

# Parse versions.yml and apply to each workflow
while IFS= read -r line; do
  if echo "$line" | grep -qE '^\s+[a-zA-Z_-]+:\s+.+'; then
    ACTION=$(echo "$line" | sed 's/^[[:space:]]*//' | cut -d: -f1 | tr -d ' ')
    VERSION=$(echo "$line" | sed 's/^[[:space:]]*//' | cut -d: -f2- | tr -d ' ')
    
    # Map action names to GitHub action references
    case "$ACTION" in
      checkout) REF="actions/checkout" ;;
      labeler) REF="actions/labeler" ;;
      rust-toolchain) REF="dtolnay/rust-toolchain" ;;
      rust-cache) REF="Swatinem/rust-cache" ;;
      upload-artifact) REF="actions/upload-artifact" ;;
      github-script) REF="actions/github-script" ;;
      auto-approve) REF="hmarr/auto-approve-action" ;;
      wait-for-checks) REF="poseidon/wait-for-status-checks" ;;
      audit-check) REF="rustsec/audit-check" ;;
      actionlint) REF="reviewdog/action-actionlint" ;;
      stale) REF="actions/stale" ;;
      release-please) REF="googleapis/release-please-action" ;;
      gitleaks) REF="gitleaks/gitleaks-action" ;;
      *) echo "  Unknown action: $ACTION"; continue ;;
    esac
    
    for f in "$WORKFLOWS_DIR"/*.yml; do
      if grep -q "$REF@" "$f" 2>/dev/null; then
        sed -i "s|$REF@.*|$REF@$VERSION|" "$f"
        echo "  $REF @ $VERSION -> $(basename $f)"
      fi
    done
  fi
done < "$VERSIONS_FILE"

echo ""
echo "Done. Updated $(grep -c "$REF@" "$WORKFLOWS_DIR"/*.yml 2>/dev/null || echo 0) references."
