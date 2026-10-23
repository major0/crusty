#!/bin/sh
# run-integration-apps.sh — build and run every example application under
# examples/apps/*. Each app is a real Crusty program that links a cargo crate;
# a build or run failure fails this script (and thus the integration gate).
#
# crustyc must be on PATH (the integration workflow builds it first; locally,
# `cargo build --release --bin crustyc` then add target/release to PATH).
#
# Runnable locally and from CI; NOT part of pre-commit.
set -eu

apps_dir="examples/apps"

if ! command -v crustyc >/dev/null 2>&1; then
    echo "run-integration-apps: crustyc not found on PATH" >&2
    echo "  build it first: cargo build --release --bin crustyc" >&2
    echo "  then: export PATH=\"\$PWD/target/release:\$PATH\"" >&2
    exit 1
fi

if [ ! -d "${apps_dir}" ]; then
    echo "run-integration-apps: no ${apps_dir} directory" >&2
    exit 1
fi

ran=0
for app in "${apps_dir}"/*/; do
    [ -f "${app}Cargo.toml" ] || continue
    ran=1
    echo "=== ${app} ==="
    # `crusty test` (crustyc test) transpiles the app's .crst and delegates to
    # the native `cargo test` harness, exposing crustyc on PATH itself (ADR-0006).
    ( cd "${app}" && crustyc test --quiet )
done

if [ "${ran}" -eq 0 ]; then
    echo "run-integration-apps: no apps found under ${apps_dir}" >&2
    exit 1
fi

echo "DONE_INTEGRATION_APPS"
