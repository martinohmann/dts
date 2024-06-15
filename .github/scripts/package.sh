#!/usr/bin/env bash
#
# Packages up releases as tar archives.

set -euo pipefail

GITHUB_OUTPUT="${GITHUB_OUTPUT:-/dev/null}"

strip_binary() {
  local target="$1"
  local bin_path="$2"
  local stripped_bin_path="$3"

  case "$target" in
    arm-unknown-linux-*)
      strip="arm-linux-gnueabihf-strip" ;;
    aarch64-unknown-linux-gnu)
      strip="aarch64-linux-gnu-strip" ;;
    *)
      strip="strip" ;;
  esac

  echo "stripping binary $bin_path -> $stripped_bin_path"

  "$strip" -o "$stripped_bin_path" "$bin_path"
}

create_package() {
  local archive_dir="$1"
  local bin_path="$2"

  echo "copying package files to $archive_dir"

  cp "$bin_path" "$archive_dir"
  cp "README.md" "LICENSE" "CHANGELOG.md" "$archive_dir"
}

create_archive() {
  local target="$1"
  local package_dir="$2"
  local package_basename="$3"
  local archive_name="$4"

  case "$target" in
    *-darwin)
      sha512sum="gsha512sum" ;;
    *)
      sha512sum="sha512sum" ;;
  esac

  pushd "$package_dir" >/dev/null || exit 1
  echo "creating archive ${package_dir}/${archive_name}"
  tar czf "$archive_name" "$package_basename"/*

  echo "creating checksum file for archive ${package_dir}/${archive_name}.sha512"
  "$sha512sum" "$archive_name" > "${archive_name}.sha512"
  popd >/dev/null || exit 1
}

package() {
  local target="$1"
  local version="$2"

  bin_name=dts
  bin_path="target/${target}/release/${bin_name}"

  if ! [ -f "$bin_path" ]; then
    echo "release binary missing, build via:"
    echo
    echo "  cargo build --release --locked --target $target"
    exit 1
  fi

  artifacts_dir=release-artifacts
  stripped_bin_path="${artifacts_dir}/${bin_name}"

  rm -rf "$artifacts_dir"
  mkdir -p "$artifacts_dir"

  strip_binary "$target" "$bin_path" "$stripped_bin_path"

  package_basename="${bin_name}-v${version}-${target}"
  archive_name="${package_basename}.tar.gz"
  package_dir="${artifacts_dir}/package"
  archive_dir="${package_dir}/${package_basename}/"
  archive_path="${package_dir}/${archive_name}"

  mkdir -p "$archive_dir"

  create_package "$archive_dir" "$stripped_bin_path"
  create_archive "$target" "$package_dir" "$package_basename" \
    "$archive_name"

  rm -rf "$archive_dir"

  # shellcheck disable=SC2129
  echo "package_dir=${package_dir}" >> "$GITHUB_OUTPUT"
  echo "archive_name=${archive_name}" >> "$GITHUB_OUTPUT"
  echo "archive_path=${archive_path}" >> "$GITHUB_OUTPUT"
}

if [ $# -lt 2 ]; then
  echo "usage: $0 <target> <version>"
  exit 1
fi

package "$@"
