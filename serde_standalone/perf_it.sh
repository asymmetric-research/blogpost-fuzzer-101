#!/bin/bash

# perf_it.sh - Performance profiling script that collects perf traces,
# collapses stacks, and generates a flame graph

# Check if target executable is provided
if [ $# -lt 1 ]; then
    echo "Usage: $0 <target_executable> [duration_seconds]"
    echo "Example: $0 ./target/fuzzer 60"
    exit 1
fi

TARGET="$1"
DURATION="${2:-60}"  # Default to 60 seconds if not specified
OUTPUT_PERF="out.perf"
OUTPUT_FOLDED="out.folded"
OUTPUT_SVG="serde.svg"

# Check if target exists and is executable
if [ ! -x "$TARGET" ]; then
    echo "Error: '$TARGET' does not exist or is not executable."
    exit 1
fi

# Check if stackcollapse.pl and flamegraph.pl exist
if [ ! -x "./stackcollapse.pl" ]; then
    echo "Error: './stackcollapse.pl' does not exist or is not executable."
    exit 1
fi

if [ ! -x "./flamegraph.pl" ]; then
    echo "Error: './flamegraph.pl' does not exist or is not executable."
    exit 1
fi

echo "Starting performance profiling of '$TARGET' for $DURATION seconds..."

# Run the target in the background
"$TARGET" &
TARGET_PID=$!

# Check if target is running
if ! ps -p $TARGET_PID > /dev/null; then
    echo "Error: Failed to start target process."
    exit 1
fi

echo "Target process running with PID: $TARGET_PID"

# Run perf record on the target process
echo "Running perf record for $DURATION seconds..."
sudo perf record -g -p $TARGET_PID -o "$OUTPUT_PERF" -- sleep $DURATION && sudo perf script > out.perf

# Kill the target process gracefully
echo "Stopping target process..."
kill -TERM $TARGET_PID

# Wait for process to terminate
wait $TARGET_PID 2>/dev/null || true

# Check if perf file was created
if [ ! -f "$OUTPUT_PERF" ]; then
    echo "Error: No perf data was collected. Check if perf is installed."
    exit 1
fi

# Run stackcollapse.pl
echo "Processing stack traces with stackcollapse.pl..."
./stackcollapse.pl "$OUTPUT_PERF" > "$OUTPUT_FOLDED"

# Check if folded file was created
if [ ! -f "$OUTPUT_FOLDED" ]; then
    echo "Error: Failed to create folded stacks file."
    exit 1
fi

# Run flamegraph.pl
echo "Generating flame graph..."
./flamegraph.pl "$OUTPUT_FOLDED" > "$OUTPUT_SVG"

# Check if SVG was created
if [ ! -f "$OUTPUT_SVG" ]; then
    echo "Error: Failed to create flame graph."
    exit 1
fi

echo "Done! Flame graph has been generated at: $OUTPUT_SVG"
echo "Performance data: $OUTPUT_PERF"
echo "Folded stacks: $OUTPUT_FOLDED"
