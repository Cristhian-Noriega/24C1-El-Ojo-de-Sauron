#!/bin/bash

# Number of drone instances
NUM_DRONES=$1

# Build all binaries
cargo build --release --manifest-path project/server/Cargo.toml
cargo build --release --manifest-path project/monitor/Cargo.toml
cargo build --release --manifest-path project/camera-system/Cargo.toml
cargo build --release --manifest-path project/drone/Cargo.toml

# Function to clean up background processes
cleanup() {
    echo "Killing background processes..."
    for pid in "${pids[@]}"; do
        kill "$pid" 2>/dev/null
    done
}

# Trap EXIT signal to ensure cleanup is called
trap cleanup EXIT

# Array to hold process IDs
pids=()

# Run the server first and wait for it to start
./project/target/release/server project/server/Settings.toml &
pids+=($!)
sleep 5

# Run the monitor next
./project/target/release/monitor project/monitor/Settings.toml &
pids+=($!)
sleep 2

# Run the camera system
./project/target/release/camera-system project/camera-system/config.json &
pids+=($!)

# Run each drone instance
for ((i=1; i<=NUM_DRONES; i++)); do
    ./project/target/release/drone project/drone/config/config_${i}.json &
    pids+=($!)
done

# Wait for all background processes to complete
wait