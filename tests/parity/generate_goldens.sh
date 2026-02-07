#!/usr/bin/env bash
#
# Generate golden output files for parity testing.
#
# This script builds the BioFabric Docker image (if needed) and runs the
# GoldenGenerator against every .sif and .gw file in tests/parity/networks/.
# Golden files are written to tests/parity/goldens/<network-name>/.
#
# Usage:
#   ./tests/parity/generate_goldens.sh            # generate all
#   ./tests/parity/generate_goldens.sh triangle    # generate one (matches name)
#   ./tests/parity/generate_goldens.sh --rebuild   # force Docker rebuild
#
# The first run takes a few minutes to compile BioFabric inside Docker.
# Subsequent runs reuse the cached image and are fast.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
NETWORKS_DIR="$SCRIPT_DIR/networks"
GOLDENS_DIR="$SCRIPT_DIR/goldens"
IMAGE_NAME="biofabric-golden"

# ---------- Parse args ----------

REBUILD=false
FILTER=""

for arg in "$@"; do
    case "$arg" in
        --rebuild)
            REBUILD=true
            ;;
        *)
            FILTER="$arg"
            ;;
    esac
done

# ---------- Build Docker image ----------

if $REBUILD || ! docker image inspect "$IMAGE_NAME" &>/dev/null; then
    echo "=== Building Docker image: $IMAGE_NAME ==="
    echo "    (This compiles BioFabric from source — takes 1-2 minutes)"
    docker build -t "$IMAGE_NAME" -f "$SCRIPT_DIR/Dockerfile" "$REPO_ROOT"
    echo "=== Docker image ready ==="
    echo
fi

# ---------- Helper function ----------

run_golden() {
    local input_file="$1"   # full path on host
    local subdir="$2"       # "sif" or "gw"
    local ext="$3"          # file extension
    local name suffix output_dir

    name="$(basename "$input_file" ".$ext")"
    suffix="${name}_default"
    [ "$subdir" = "gw" ] && suffix="${name}_gw_default"
    output_dir="$GOLDENS_DIR/$suffix"

    # If a filter is set, skip non-matching networks
    if [ -n "$FILTER" ] && [ "$name" != "$FILTER" ]; then
        return 1  # signal: skipped
    fi

    mkdir -p "$output_dir"

    if docker run --rm \
        -v "$NETWORKS_DIR/$subdir:/networks:ro" \
        -v "$output_dir:/goldens" \
        "$IMAGE_NAME" \
        "/networks/$(basename "$input_file")" "/goldens" \
        > "$output_dir/generator.log" 2>&1; then
        echo "OK"
        return 0
    else
        echo "FAILED (see $output_dir/generator.log)"
        return 2  # signal: failed
    fi
}

# ---------- Generate golden files ----------

mkdir -p "$GOLDENS_DIR"

TOTAL=0
OK=0
FAIL=0

echo "--- SIF networks ---"
for f in "$NETWORKS_DIR"/sif/*.sif; do
    [ -f "$f" ] || continue
    name="$(basename "$f" .sif)"
    [ -n "$FILTER" ] && [ "$name" != "$FILTER" ] && continue
    TOTAL=$((TOTAL + 1))
    echo -n "[$TOTAL] $name.sif ... "
    rc=0; run_golden "$f" sif sif || rc=$?
    if [ "$rc" -eq 0 ]; then OK=$((OK + 1));
    elif [ "$rc" -eq 2 ]; then FAIL=$((FAIL + 1)); fi
done

echo
echo "--- GW networks ---"
for f in "$NETWORKS_DIR"/gw/*.gw; do
    [ -f "$f" ] || continue
    name="$(basename "$f" .gw)"
    [ -n "$FILTER" ] && [ "$name" != "$FILTER" ] && continue
    TOTAL=$((TOTAL + 1))
    echo -n "[$TOTAL] $name.gw ... "
    rc=0; run_golden "$f" gw gw || rc=$?
    if [ "$rc" -eq 0 ]; then OK=$((OK + 1));
    elif [ "$rc" -eq 2 ]; then FAIL=$((FAIL + 1)); fi
done

echo
echo "=== Results: $OK/$TOTAL passed, $FAIL failed ==="
if [ $FAIL -gt 0 ]; then
    exit 1
fi
