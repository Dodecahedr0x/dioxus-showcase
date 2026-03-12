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
    next
  }

  /^\[workspace\.dependencies\]$/ {
    in_workspace_package = 0
    in_workspace_dependencies = 1
    next
  }

  /^\[/ {
    in_workspace_package = 0
    in_workspace_dependencies = 0
  }

  in_workspace_package && /^version = "/ {
    if ($0 != "version = \"" version "\"") {
      print "[verify-workspace-version] workspace.package version does not match " version > "/dev/stderr"
      exit 1
    }

    saw_workspace_version = 1
    next
  }

  in_workspace_dependencies && /^dioxus-showcase-core = \{/ {
    if ($0 !~ "version = \"" version "\"") {
      print "[verify-workspace-version] dioxus-showcase-core dependency version does not match " version > "/dev/stderr"
      exit 1
    }

    saw_core_dep = 1
    next
  }

  in_workspace_dependencies && /^dioxus-showcase-macros = \{/ {
    if ($0 !~ "version = \"" version "\"") {
      print "[verify-workspace-version] dioxus-showcase-macros dependency version does not match " version > "/dev/stderr"
      exit 1
    }

    saw_macros_dep = 1
    next
  }

  in_workspace_dependencies && /^dioxus-showcase = \{/ {
    if ($0 !~ "version = \"" version "\"") {
      print "[verify-workspace-version] dioxus-showcase dependency version does not match " version > "/dev/stderr"
      exit 1
    }

    saw_showcase_dep = 1
    next
  }

  END {
    if (!saw_workspace_version) {
      print "[verify-workspace-version] failed to find [workspace.package] version" > "/dev/stderr"
      exit 1
    }

    if (!saw_core_dep || !saw_macros_dep || !saw_showcase_dep) {
      print "[verify-workspace-version] failed to find one or more internal workspace dependency versions" > "/dev/stderr"
      exit 1
    }
  }
' "$CARGO_TOML"

echo "Verified workspace manifest version $VERSION"
