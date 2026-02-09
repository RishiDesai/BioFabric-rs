"""
Speed benchmark tests: Rust biofabric-rs vs Java BioFabric.

Compares wall-clock time for the full layout pipeline (parse -> layout ->
serialize) on large networks.  Correctness is verified separately by the
parity test suite; these tests only measure performance.

Designed to run inside a standardized container with fixed resources
(CPUs, RAM) for reproducible measurements.  Can also run locally if Java
BioFabric classes and the Rust binary are available.

== Methodology ==

  1. External wall-clock timing (time.monotonic) — cannot be gamed
  2. Multiple trials per binary (default: 5)
  3. Median aggregation — filters scheduling jitter / GC pauses
  4. Escalating thresholds — Rust's advantage grows with network size
     because cache-friendliness, zero-copy, and no-GC compound at scale

Java is cold-started each trial (new JVM process).  This is intentional:
it measures real user-facing latency, not warmed-up throughput.

== Environment variables ==

  BENCH_TRIALS       Number of trials per binary (default: 5)
  JAVA_CLASSPATH     Path to compiled Java BioFabric classes
                     (default: /build/classes)
  JAVA_XMX           Java max heap size (default: 4g)
  RUST_BINARY        Path to compiled Rust biofabric binary
                     (default: target/release/biofabric)
  NETWORKS_DIR       Path to test networks directory
                     (default: tests/parity/networks)
  BENCH_OUTPUT_DIR   Temp directory for benchmark outputs
                     (default: /tmp/biofabric_bench)
  LOGS_DIR           Directory for performance JSON logs
                     (default: $BENCH_OUTPUT_DIR/logs)
"""

import json
import os
import shutil
import statistics
import subprocess
import tempfile
import time

import pytest

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

BENCH_TRIALS = int(os.getenv("BENCH_TRIALS", "5"))

JAVA_CLASSPATH = os.getenv("JAVA_CLASSPATH", "/build/classes")
JAVA_XMX = os.getenv("JAVA_XMX", "4g")
RUST_BINARY = os.getenv("RUST_BINARY", "target/release/biofabric")

# Network directory: defaults to tests/parity/networks/ (shared with parity tests)
NETWORKS_DIR = os.getenv("NETWORKS_DIR", "tests/parity/networks")

BENCH_OUTPUT_DIR = os.getenv("BENCH_OUTPUT_DIR", "/tmp/biofabric_bench")
LOGS_DIR = os.getenv("LOGS_DIR", os.path.join(BENCH_OUTPUT_DIR, "logs"))

# ---------------------------------------------------------------------------
# Speedup thresholds
#
# Conservative initial values.  Calibrate after first measurements in the
# target container environment.
#
# Rust's advantage compounds with network scale:
#   - No JVM startup / class loading / JIT compilation overhead
#   - No GC pauses (BioFabric allocates heavily during layout)
#   - Cache-friendly data structures (contiguous memory, no pointer chasing)
#   - Zero-copy SIF/GW parsing
#   - No boxing overhead for primitives
#
# Thresholds escalate by network size because these advantages matter more
# as working-set size exceeds L2/L3 cache.
# ---------------------------------------------------------------------------

# SC.sif: ~6,600 nodes, ~77K edges (S. cerevisiae, dense)
REQUIRED_SPEEDUP_SC = 2.0


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _ensure_dirs():
    os.makedirs(BENCH_OUTPUT_DIR, exist_ok=True)
    os.makedirs(LOGS_DIR, exist_ok=True)


def _network_path(filename):
    """Resolve a network filename to its full path under NETWORKS_DIR."""
    if filename.endswith(".sif"):
        return os.path.join(NETWORKS_DIR, "sif", filename)
    elif filename.endswith(".gw"):
        return os.path.join(NETWORKS_DIR, "gw", filename)
    else:
        return os.path.join(NETWORKS_DIR, filename)


def _run_java_wallclock(network_file, extra_args=None):
    """Run Java BioFabric (GoldenGenerator) and return wall-clock seconds.

    Uses time.monotonic() for external timing so the result cannot be
    gamed by tampering with Java's self-reported output.
    """
    _ensure_dirs()
    output_dir = tempfile.mkdtemp(dir=BENCH_OUTPUT_DIR, prefix="java_")
    net_path = _network_path(network_file)

    cmd = [
        "java",
        "-Djava.awt.headless=true",
        f"-Xmx{JAVA_XMX}",
        "-cp", JAVA_CLASSPATH,
        "GoldenGenerator",
        net_path, output_dir,
    ]
    if extra_args:
        cmd.extend(extra_args)

    start = time.monotonic()
    result = subprocess.run(
        cmd, capture_output=True, text=True, timeout=600,
    )
    elapsed = time.monotonic() - start

    assert result.returncode == 0, (
        f"Java BioFabric exited with code {result.returncode}.\n"
        f"--- stderr (last 2000 chars) ---\n{result.stderr[-2000:]}"
    )

    # Clean up temp output
    shutil.rmtree(output_dir, ignore_errors=True)
    return elapsed


def _run_rust_wallclock(network_file, extra_args=None):
    """Run Rust biofabric layout and return wall-clock seconds.

    Uses time.monotonic() for external timing.
    """
    _ensure_dirs()
    output_file = tempfile.mktemp(dir=BENCH_OUTPUT_DIR, suffix=".bif")
    net_path = _network_path(network_file)

    cmd = [RUST_BINARY, "layout", net_path, "-o", output_file]
    if extra_args:
        cmd.extend(extra_args)

    start = time.monotonic()
    result = subprocess.run(
        cmd, capture_output=True, text=True, timeout=600,
    )
    elapsed = time.monotonic() - start

    assert result.returncode == 0, (
        f"Rust biofabric exited with code {result.returncode}.\n"
        f"--- stderr (last 2000 chars) ---\n{result.stderr[-2000:]}"
    )

    # Clean up temp output
    if os.path.exists(output_file):
        os.remove(output_file)
    return elapsed


def _log_speed_result(label, java_median, rust_median, speedup,
                      required_speedup, java_trials, rust_trials):
    """Log speed-test results to performance.json for later analysis."""
    _ensure_dirs()
    perf_file = os.path.join(LOGS_DIR, "performance.json")
    perf_data = {}
    if os.path.exists(perf_file):
        try:
            with open(perf_file) as fh:
                perf_data = json.load(fh)
        except (json.JSONDecodeError, OSError):
            pass

    perf_data[label] = {
        "java_median_secs": round(java_median, 4),
        "rust_median_secs": round(rust_median, 4),
        "speedup": round(speedup, 2),
        "required_speedup": required_speedup,
        "passed": speedup >= required_speedup,
        "trials": BENCH_TRIALS,
        "java_all_secs": [round(t, 4) for t in java_trials],
        "rust_all_secs": [round(t, 4) for t in rust_trials],
    }

    with open(perf_file, "w") as fh:
        json.dump(perf_data, fh, indent=2)


def _run_speed_test(label, network_file, required_speedup,
                    java_extra_args=None, rust_extra_args=None):
    """Generic speed-test driver.

    Runs both Java and Rust binaries BENCH_TRIALS times each, computes
    median wall-clock time, logs results, and asserts the required
    speedup is met.

    Returns (speedup, java_median, rust_median).
    """
    # -- Java trials --
    java_samples = []
    for i in range(BENCH_TRIALS):
        elapsed = _run_java_wallclock(network_file, java_extra_args)
        java_samples.append(elapsed)

    # -- Rust trials --
    rust_samples = []
    for i in range(BENCH_TRIALS):
        elapsed = _run_rust_wallclock(network_file, rust_extra_args)
        rust_samples.append(elapsed)

    java_median = statistics.median(java_samples)
    rust_median = statistics.median(rust_samples)
    speedup = java_median / rust_median

    # Log results
    _log_speed_result(
        label, java_median, rust_median, speedup,
        required_speedup, java_samples, rust_samples,
    )

    # Print for visibility in test output
    print(f"\n{'='*60}")
    print(f"  {label}")
    print(f"  Network:  {network_file}")
    print(f"  Trials:   {BENCH_TRIALS}")
    print(f"  Java:     {java_median:.3f}s (median)  "
          f"[{min(java_samples):.3f}s .. {max(java_samples):.3f}s]")
    print(f"  Rust:     {rust_median:.3f}s (median)  "
          f"[{min(rust_samples):.3f}s .. {max(rust_samples):.3f}s]")
    print(f"  Speedup:  {speedup:.2f}x  (required: >= {required_speedup}x)")
    print(f"  Result:   {'PASS' if speedup >= required_speedup else 'FAIL'}")
    print(f"{'='*60}\n")

    assert speedup >= required_speedup, (
        f"Rust is not fast enough ({label}).  "
        f"Required: >= {required_speedup:.1f}x faster than Java, "
        f"got: {speedup:.2f}x.  "
        f"(Java median {java_median:.3f}s, Rust median {rust_median:.3f}s, "
        f"trials: {BENCH_TRIALS})"
    )

    return speedup, java_median, rust_median


# ---------------------------------------------------------------------------
# Prerequisite checks
# ---------------------------------------------------------------------------

def test_java_available():
    """Java BioFabric classes must be available."""
    assert os.path.isdir(JAVA_CLASSPATH), (
        f"Java BioFabric classes not found at {JAVA_CLASSPATH}.  "
        f"Build the Docker image first: "
        f"docker build -t biofabric-golden -f tests/parity/Dockerfile ."
    )


def test_rust_binary_available():
    """Rust biofabric binary must be compiled."""
    assert os.path.isfile(RUST_BINARY) or shutil.which(RUST_BINARY), (
        f"Rust biofabric binary not found at {RUST_BINARY}.  "
        f"Compile with: cargo build --release -p biofabric"
    )


# ---------------------------------------------------------------------------
# Speed tests — default layout, large networks only
#
# Networks are ordered by size (ascending).  Thresholds escalate because
# Rust's advantage grows with network scale.
# ---------------------------------------------------------------------------

def test_speed_SC_default():
    """SC.sif (~6.6K nodes, ~77K edges): default layout.

    Dense S. cerevisiae network — more edges than nodes.  Working set
    exceeds L2 cache; this is where cache-friendly data structures
    (contiguous adjacency lists, flat index maps) start to dominate.
    """
    _run_speed_test(
        "speed_SC_default",
        "SC.sif",
        REQUIRED_SPEEDUP_SC,
    )


# ---------------------------------------------------------------------------
# Speed tests — alternative layouts (large networks)
#
# These test layout algorithms beyond the default BFS.  Each exercises
# different computational patterns (similarity matrices, hierarchical
# grouping) where Rust's advantages may differ.
# ---------------------------------------------------------------------------

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

def test_zz_print_summary():
    """Print performance summary from all speed tests (runs last)."""
    perf_file = os.path.join(LOGS_DIR, "performance.json")
    if not os.path.exists(perf_file):
        print("No performance data found.")
        return

    with open(perf_file) as fh:
        perf_data = json.load(fh)

    print(f"\n{'='*72}")
    print(f"  PERFORMANCE SUMMARY")
    print(f"{'='*72}")
    print(f"  {'Test':<35} {'Java':>8} {'Rust':>8} {'Speedup':>8} {'Req':>6} {'Result':>6}")
    print(f"  {'-'*35} {'-'*8} {'-'*8} {'-'*8} {'-'*6} {'-'*6}")

    for label, data in sorted(perf_data.items()):
        java_s = f"{data['java_median_secs']:.3f}s"
        rust_s = f"{data['rust_median_secs']:.3f}s"
        speedup_s = f"{data['speedup']:.2f}x"
        req_s = f"{data['required_speedup']:.1f}x"
        result_s = "PASS" if data["passed"] else "FAIL"
        print(f"  {label:<35} {java_s:>8} {rust_s:>8} {speedup_s:>8} {req_s:>6} {result_s:>6}")

    print(f"{'='*72}\n")
