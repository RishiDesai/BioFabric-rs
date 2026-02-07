#!/usr/bin/env bash
#
# Generate golden output files for parity testing.
#
# This script builds the BioFabric Docker image (if needed) and runs the
# GoldenGenerator against every .sif and .gw file in tests/parity/networks/.
# Each network is tested with multiple configuration variants:
#
#   1. Default layout + shadows ON   (baseline)
#   2. Default layout + shadows OFF  (shadow toggle test)
#   3. Link grouping variants        (for multi-relation networks only)
#
# Golden files are written to tests/parity/goldens/<variant-name>/.
#
# Usage:
#   ./tests/parity/generate_goldens.sh             # generate all
#   ./tests/parity/generate_goldens.sh triangle     # generate one (matches name)
#   ./tests/parity/generate_goldens.sh --rebuild    # force Docker rebuild
#   ./tests/parity/generate_goldens.sh --shadows-only  # only shadows-on variants
#   ./tests/parity/generate_goldens.sh --no-shadows-only # only shadows-off variants
#   ./tests/parity/generate_goldens.sh --groups-only  # only link grouping variants
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
ONLY_SHADOWS_ON=false
ONLY_SHADOWS_OFF=false
ONLY_GROUPS=false

for arg in "$@"; do
    case "$arg" in
        --rebuild)        REBUILD=true ;;
        --shadows-only)   ONLY_SHADOWS_ON=true ;;
        --no-shadows-only) ONLY_SHADOWS_OFF=true ;;
        --groups-only)    ONLY_GROUPS=true ;;
        *)                FILTER="$arg" ;;
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

# run_one <input_file> <subdir> <ext> <output_suffix> [extra_flags...]
run_one() {
    local input_file="$1"
    local subdir="$2"
    local ext="$3"
    local output_suffix="$4"
    shift 4
    local extra_flags=("$@")

    local output_dir="$GOLDENS_DIR/$output_suffix"
    mkdir -p "$output_dir"

    local docker_args=(
        docker run --rm
        -v "$NETWORKS_DIR/$subdir:/networks:ro"
        -v "$output_dir:/goldens"
        "$IMAGE_NAME"
        "/networks/$(basename "$input_file")" "/goldens"
    )

    # Append extra flags (e.g., --no-shadows, --link-groups, etc.)
    if [ ${#extra_flags[@]} -gt 0 ]; then
        docker_args+=("${extra_flags[@]}")
    fi

    if "${docker_args[@]}" > "$output_dir/generator.log" 2>&1; then
        echo "OK"
        return 0
    else
        echo "FAILED (see $output_dir/generator.log)"
        return 2
    fi
}

# ---------- Generate golden files ----------

mkdir -p "$GOLDENS_DIR"

TOTAL=0
OK=0
FAIL=0

# =========================================================================
# Phase 1: Default layout + shadows ON (baseline)
# =========================================================================

if ! $ONLY_SHADOWS_OFF && ! $ONLY_GROUPS; then
    echo "=== Phase 1: Default layout + shadows ON ==="
    echo

    echo "--- SIF networks ---"
    for f in "$NETWORKS_DIR"/sif/*.sif; do
        [ -f "$f" ] || continue
        name="$(basename "$f" .sif)"
        [ -n "$FILTER" ] && [ "$name" != "$FILTER" ] && continue
        TOTAL=$((TOTAL + 1))
        echo -n "  [$TOTAL] ${name}_default ... "
        rc=0; run_one "$f" sif sif "${name}_default" || rc=$?
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
        echo -n "  [$TOTAL] ${name}_gw_default ... "
        rc=0; run_one "$f" gw gw "${name}_gw_default" || rc=$?
        if [ "$rc" -eq 0 ]; then OK=$((OK + 1));
        elif [ "$rc" -eq 2 ]; then FAIL=$((FAIL + 1)); fi
    done
    echo
fi

# =========================================================================
# Phase 2: Default layout + shadows OFF
# =========================================================================

if ! $ONLY_SHADOWS_ON && ! $ONLY_GROUPS; then
    echo "=== Phase 2: Default layout + shadows OFF ==="
    echo

    echo "--- SIF networks (no shadows) ---"
    for f in "$NETWORKS_DIR"/sif/*.sif; do
        [ -f "$f" ] || continue
        name="$(basename "$f" .sif)"
        [ -n "$FILTER" ] && [ "$name" != "$FILTER" ] && continue
        TOTAL=$((TOTAL + 1))
        echo -n "  [$TOTAL] ${name}_noshadow ... "
        rc=0; run_one "$f" sif sif "${name}_noshadow" --no-shadows || rc=$?
        if [ "$rc" -eq 0 ]; then OK=$((OK + 1));
        elif [ "$rc" -eq 2 ]; then FAIL=$((FAIL + 1)); fi
    done

    echo
    echo "--- GW networks (no shadows) ---"
    for f in "$NETWORKS_DIR"/gw/*.gw; do
        [ -f "$f" ] || continue
        name="$(basename "$f" .gw)"
        [ -n "$FILTER" ] && [ "$name" != "$FILTER" ] && continue
        TOTAL=$((TOTAL + 1))
        echo -n "  [$TOTAL] ${name}_gw_noshadow ... "
        rc=0; run_one "$f" gw gw "${name}_gw_noshadow" --no-shadows || rc=$?
        if [ "$rc" -eq 0 ]; then OK=$((OK + 1));
        elif [ "$rc" -eq 2 ]; then FAIL=$((FAIL + 1)); fi
    done
    echo
fi

# =========================================================================
# Phase 3: Link grouping variants (multi-relation networks only)
# =========================================================================

if ! $ONLY_SHADOWS_ON && ! $ONLY_SHADOWS_OFF; then
    echo "=== Phase 3: Link grouping variants ==="
    echo

    # multi_relation.sif has relations: pp, pd, gi
    if [ -z "$FILTER" ] || [ "$FILTER" = "multi_relation" ]; then
        TOTAL=$((TOTAL + 1))
        echo -n "  [$TOTAL] multi_relation_pernode ... "
        rc=0; run_one "$NETWORKS_DIR/sif/multi_relation.sif" sif sif \
            "multi_relation_pernode" \
            --link-groups "pp,pd,gi" --group-mode per_node || rc=$?
        if [ "$rc" -eq 0 ]; then OK=$((OK + 1));
        elif [ "$rc" -eq 2 ]; then FAIL=$((FAIL + 1)); fi

        TOTAL=$((TOTAL + 1))
        echo -n "  [$TOTAL] multi_relation_pernetwork ... "
        rc=0; run_one "$NETWORKS_DIR/sif/multi_relation.sif" sif sif \
            "multi_relation_pernetwork" \
            --link-groups "pp,pd,gi" --group-mode per_network || rc=$?
        if [ "$rc" -eq 0 ]; then OK=$((OK + 1));
        elif [ "$rc" -eq 2 ]; then FAIL=$((FAIL + 1)); fi
    fi

    # directed_triangle.gw has relations: activates, inhibits
    if [ -z "$FILTER" ] || [ "$FILTER" = "directed_triangle" ]; then
        TOTAL=$((TOTAL + 1))
        echo -n "  [$TOTAL] directed_triangle_gw_pernode ... "
        rc=0; run_one "$NETWORKS_DIR/gw/directed_triangle.gw" gw gw \
            "directed_triangle_gw_pernode" \
            --link-groups "activates,inhibits" --group-mode per_node || rc=$?
        if [ "$rc" -eq 0 ]; then OK=$((OK + 1));
        elif [ "$rc" -eq 2 ]; then FAIL=$((FAIL + 1)); fi

        TOTAL=$((TOTAL + 1))
        echo -n "  [$TOTAL] directed_triangle_gw_pernetwork ... "
        rc=0; run_one "$NETWORKS_DIR/gw/directed_triangle.gw" gw gw \
            "directed_triangle_gw_pernetwork" \
            --link-groups "activates,inhibits" --group-mode per_network || rc=$?
        if [ "$rc" -eq 0 ]; then OK=$((OK + 1));
        elif [ "$rc" -eq 2 ]; then FAIL=$((FAIL + 1)); fi
    fi
    echo
fi

# =========================================================================
# Summary
# =========================================================================

echo "=== Results: $OK/$TOTAL passed, $FAIL failed ==="
if [ $FAIL -gt 0 ]; then
    exit 1
fi
