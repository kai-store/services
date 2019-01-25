#!/bin/bash

set -x -e

./build.sh --release --strip
cp target/armv7-linux-androideabi/release/metrics_daemon prebuilts/armv7-linux-androideabi/metrics_daemon
