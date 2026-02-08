#!/usr/bin/env bash
#
# Run speed benchmarks: Rust biofabric-rs vs Java BioFabric.
#
# This script handles both local development and container environments:
#
#   Local:     Builds the Docker image, compiles Rust in release mode,
#              copies Java classes out of Docker, and runs pytest.
#
#   Container: Assumes both binaries are available and runs pytest directly.
#
# Usage:
#   ./tests/bench/run_bench.sh                    # run all speed tests
#   ./tests/bench/run_bench.sh -k SC              # run one test
#   ./tests/bench/run_bench.sh --trials 3          # fewer trials (faster)
#   ./tests/bench/run_bench.sh --container         # skip Docker/compile steps
#   ./tests/bench/run_bench.sh --dry-run           # print config, don't run
#
# Environment:
#   BENCH_TRIALS       Number of trials per binary (default: 5)
#   JAVA_CLASSPATH     Path to Java BioFabric classes
#   RUST_BINARY        Path to Rust biofabric binary
#   NETWORKS_DIR       Path to test networks
#   BENCH_OUTPUT_DIR   Temp directory for outputs
#   LOGS_DIR           Directory for performance JSON logs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
IMAGE_NAME="biofabric-golden"

# ---------- Parse args ----------

CONTAINER_MODE=false
DRY_RUN=false
TRIALS=""
PYTEST_ARGS=()

while [ $# -gt 0 ]; do
    case "$1" in
        --container)    CONTAINER_MODE=true; shift ;;
        --dry-run)      DRY_RUN=true; shift ;;
        --trials)       TRIALS="$2"; shift 2 ;;
        *)              PYTEST_ARGS+=("$1"); shift ;;
    esac
done

# ---------- Resolve paths ----------

# Networks live alongside parity tests (shared)
NETWORKS_DIR="${NETWORKS_DIR:-$REPO_ROOT/tests/parity/networks}"
BENCH_OUTPUT_DIR="${BENCH_OUTPUT_DIR:-/tmp/biofabric_bench}"
LOGS_DIR="${LOGS_DIR:-$BENCH_OUTPUT_DIR/logs}"

mkdir -p "$BENCH_OUTPUT_DIR" "$LOGS_DIR"

# ---------- Local mode: build Docker + extract Java classes ----------

if ! $CONTAINER_MODE; then
    echo "=== Local development mode ==="
    echo

    # 1. Build Docker image (reuses the parity test image)
    if ! docker image inspect "$IMAGE_NAME" &>/dev/null; then
        echo "--- Building Docker image: $IMAGE_NAME ---"
        echo "    (Compiles BioFabric from source — takes 1-2 minutes)"
        docker build -t "$IMAGE_NAME" -f "$REPO_ROOT/tests/parity/Dockerfile" "$REPO_ROOT"
        echo "--- Docker image ready ---"
        echo
    else
        echo "--- Docker image $IMAGE_NAME already exists (use --rebuild to force) ---"
        echo
    fi

    # 2. Extract Java classes from Docker image so pytest can call java directly
    JAVA_CLASSPATH="${JAVA_CLASSPATH:-$BENCH_OUTPUT_DIR/java_classes}"
    if [ ! -d "$JAVA_CLASSPATH" ]; then
        echo "--- Extracting Java classes from Docker image ---"
        container_id=$(docker create "$IMAGE_NAME")
        mkdir -p "$JAVA_CLASSPATH"
        docker cp "$container_id:/build/classes/." "$JAVA_CLASSPATH/"
        docker rm "$container_id" > /dev/null
        echo "    Classes extracted to $JAVA_CLASSPATH"
        echo
    fi

    # 3. Compile Rust binary in release mode
    RUST_BINARY="${RUST_BINARY:-$REPO_ROOT/target/release/biofabric}"
    if [ ! -f "$RUST_BINARY" ] || [ "$REPO_ROOT/crates/cli/src/main.rs" -nt "$RUST_BINARY" ]; then
        echo "--- Compiling Rust biofabric (release mode) ---"
        cargo build --release -p biofabric-cli --manifest-path "$REPO_ROOT/Cargo.toml"
        echo "--- Rust binary ready ---"
        echo
    else
        echo "--- Rust binary up to date ---"
        echo
    fi

    # 4. Check that Java is available
    if ! command -v java &>/dev/null; then
        echo "ERROR: Java not found. Install JDK 11+ or use --container mode."
        echo "       On macOS: brew install openjdk@11"
        exit 1
    fi
else
    echo "=== Container mode ==="
    echo

    # In container, expect both binaries at default locations
    JAVA_CLASSPATH="${JAVA_CLASSPATH:-/build/classes}"
    RUST_BINARY="${RUST_BINARY:-biofabric}"
fi

# ---------- Install pytest if needed ----------

if ! command -v pytest &>/dev/null; then
    echo "--- Installing pytest ---"
    pip install --quiet pytest 2>/dev/null || pip3 install --quiet pytest 2>/dev/null
    echo
fi

# ---------- Print configuration ----------

echo "=== Benchmark Configuration ==="
echo "  Networks:       $NETWORKS_DIR"
echo "  Java classpath: $JAVA_CLASSPATH"
echo "  Rust binary:    $RUST_BINARY"
echo "  Output dir:     $BENCH_OUTPUT_DIR"
echo "  Logs dir:       $LOGS_DIR"
echo "  Trials:         ${TRIALS:-$BENCH_TRIALS}"
echo

if $DRY_RUN; then
    echo "(dry run — exiting)"
    exit 0
fi

# ---------- Run benchmarks ----------

export JAVA_CLASSPATH
export RUST_BINARY
export NETWORKS_DIR
export BENCH_OUTPUT_DIR
export LOGS_DIR
[ -n "$TRIALS" ] && export BENCH_TRIALS="$TRIALS"

echo "=== Running speed benchmarks ==="
echo

pytest "$SCRIPT_DIR/test_speed.py" \
    -v \
    --tb=short \
    -s \
    "${PYTEST_ARGS[@]+"${PYTEST_ARGS[@]}"}"

echo
echo "=== Performance log: $LOGS_DIR/performance.json ==="
if [ -f "$LOGS_DIR/performance.json" ]; then
    cat "$LOGS_DIR/performance.json"
fi
