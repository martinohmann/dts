#!/usr/bin/env bash
#
# Publishes crates to crates.io.
#
# The scripts accepts additional arguments for `cargo publish`.

set -euo pipefail

# Dependencies need to be published first.
declare -a crates=("dts-core" "dts")

declare -a log_files=()

cleanup() {
  for log_file in "${log_files[@]}"; do
    rm -f "$log_file"
  done
}

publish_crate() {
  local crate="$1"; shift
  local cargo_args=("$@")
  local log_file
  local -i ret=0
  local -i attempt_num=1
  local -i max_attempts=10

  while true; do
    log_file="$(mktemp -t cargo-log.XXXXXXX)"
    log_files+=("$log_file")

    set +e
    cargo publish --package "$crate" "${cargo_args[@]}" 2>&1 | tee "$log_file"
    ret=$?
    set -e

    if [ $ret -eq 0 ] || grep -q "already uploaded" "$log_file"; then
      # All good.
      return 0
    fi

    # Only retry version requirement issues. This may mean that a just
    # uploaded crate version is not visible to cargo yet.
    if ! grep -q "failed to select a version for the requirement" "$log_file"; then
      return $ret
    fi

    if [ $attempt_num -ge $max_attempts ]; then
      break
    fi

    echo "Retrying in $attempt_num seconds..."
    sleep $((attempt_num++))
  done

  return $ret
}

publish() {
  for crate in "${crates[@]}"; do
    publish_crate "$crate" "$@"
  done
}

trap cleanup EXIT

publish "$@"
