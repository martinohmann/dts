#!/usr/bin/env bash

set -euo pipefail

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
  local pkg_dir="$1"
  local pkg_name="$2"
  local pkg_basename="$3"

  echo "creating archive ${pkg_dir}/${pkg_name}"

  pushd "${pkg_dir}/" >/dev/null || exit 1
  tar czf "$pkg_name" "$pkg_basename"/*
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

  pkg_basename="${bin_name}-v${version}-${target}"
  pkg_name="${pkg_basename}.tar.gz"
  pkg_dir="${artifacts_dir}/package"
  archive_dir="${pkg_dir}/${pkg_basename}/"

  mkdir -p "$archive_dir"

  create_package "$archive_dir" "$stripped_bin_path"
  create_archive "$pkg_dir" "$pkg_name" "$pkg_basename"

  echo ::set-output name=pkg_name::"${pkg_name}"
  echo ::set-output name=pkg_path::"${pkg_dir}/${pkg_name}"
}

if [ $# -lt 2 ]; then
  echo "usage: $0 <target> <version>"
  exit 1
fi

package "$@"
