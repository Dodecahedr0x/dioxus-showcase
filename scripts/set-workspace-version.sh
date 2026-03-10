#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
CARGO_TOML="${2:-Cargo.toml}"

if [[ -z "$VERSION" ]]; then
  echo "usage: $0 <version> [Cargo.toml path]" >&2
  exit 1
fi

if [[ ! -f "$CARGO_TOML" ]]; then
  echo "Cargo manifest not found: $CARGO_TOML" >&2
  exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  echo "Invalid semver version: $VERSION" >&2
  exit 1
fi

TMP_FILE="$(mktemp)"
trap 'rm -f "$TMP_FILE"' EXIT

awk -v version="$VERSION" '
  BEGIN {
    in_workspace_package = 0
    in_workspace_dependencies = 0
    saw_workspace_version = 0
    saw_core_dep = 0
    saw_macros_dep = 0
    saw_showcase_dep = 0
  }

  /^\[workspace\.package\]$/ {
    in_workspace_package = 1
    in_workspace_dependencies = 0
    print
    next
  }

  /^\[workspace\.dependencies\]$/ {
    in_workspace_package = 0
    in_workspace_dependencies = 1
    print
    next
  }

  /^\[/ {
    in_workspace_package = 0
    in_workspace_dependencies = 0
  }

  {
    line = $0

    if (in_workspace_package && line ~ /^version = "/) {
      print "version = \"" version "\""
      saw_workspace_version = 1
      next
    }

    if (in_workspace_dependencies && line ~ /^dioxus-showcase-core = \{/) {
      gsub(/version = "[^"]+"/, "version = \"" version "\"", line)
      print line
      saw_core_dep = 1
      next
    }

    if (in_workspace_dependencies && line ~ /^dioxus-showcase-macros = \{/) {
      gsub(/version = "[^"]+"/, "version = \"" version "\"", line)
      print line
      saw_macros_dep = 1
      next
    }

    if (in_workspace_dependencies && line ~ /^dioxus-showcase = \{/) {
      gsub(/version = "[^"]+"/, "version = \"" version "\"", line)
      print line
      saw_showcase_dep = 1
      next
    }

    print line
  }

  END {
    if (!saw_workspace_version) {
      print "[set-workspace-version] failed to update [workspace.package] version" > "/dev/stderr"
      exit 1
    }

    if (!saw_core_dep || !saw_macros_dep || !saw_showcase_dep) {
      print "[set-workspace-version] failed to update one or more internal workspace dependency versions" > "/dev/stderr"
      exit 1
    }
  }
' "$CARGO_TOML" > "$TMP_FILE"

mv "$TMP_FILE" "$CARGO_TOML"

echo "Updated workspace manifest to version $VERSION"
