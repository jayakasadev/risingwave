#!/usr/bin/env bash

# Exits as soon as any line fails.
set -euo pipefail

source ci/scripts/common.sh

while getopts 'p:' opt; do
    case ${opt} in
        p )
            profile=$OPTARG
            ;;
        \? )
            echo "Invalid Option: -$OPTARG" 1>&2
            exit 1
            ;;
        : )
            echo "Invalid option: $OPTARG requires an argument" 1>&2
            ;;
    esac
done
shift $((OPTIND -1))

download_and_prepare_rw "$profile" common

echo "--- Download artifacts"
download-and-decompress-artifact e2e_test_generated ./

start_cluster() {
    echo "--- Start cluster"
    risedev ci-start ci-3streaming-2serving-3fe
}

kill_cluster() {
    echo "--- Kill cluster"
    risedev ci-kill
}

host_args=(-h localhost -p 4565 -h localhost -p 4566 -h localhost -p 4567)

echo "--- e2e, parallel, streaming"
RUST_LOG="info,risingwave_stream=info,risingwave_batch=info,risingwave_storage=info,risingwave_storage::hummock::compactor::compactor_runner=warn" \
start_cluster
risedev slt "${host_args[@]}" -d dev './e2e_test/streaming/**/*.slt' -j 16 --junit "parallel-streaming-${profile}" --label "parallel"
kill_cluster

echo "--- e2e, parallel, batch"
RUST_LOG="info,risingwave_stream=info,risingwave_batch=info,risingwave_storage=info,risingwave_storage::hummock::compactor::compactor_runner=warn" \
start_cluster
# Exclude files that contain ALTER SYSTEM commands
find ./e2e_test/ddl -name "*.slt" -type f -exec grep -L "ALTER SYSTEM" {} \; | xargs -r sqllogictest "${host_args[@]}" -d dev --junit "parallel-batch-ddl-${profile}" --label "parallel"
risedev slt "${host_args[@]}" -d dev './e2e_test/visibility_mode/*.slt' -j 16 --junit "parallel-batch-${profile}" --label "parallel"
kill_cluster

echo "--- e2e, parallel, generated"
RUST_LOG="info,risingwave_stream=info,risingwave_batch=info,risingwave_storage=info,risingwave_storage::hummock::compactor::compactor_runner=warn" \
start_cluster
risedev slt "${host_args[@]}" -d dev './e2e_test/generated/**/*.slt' -j 16 --junit "parallel-generated-${profile}" --label "parallel"
kill_cluster
