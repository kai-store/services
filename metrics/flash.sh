#!/bin/bash

set -e

./build.sh --release

arm-linux-androideabi-strip target/armv7-linux-androideabi/release/metrics_daemon

adb shell mkdir -p /system/kaios

adb push target/armv7-linux-androideabi/release/metrics_daemon /system/kaios/metricsd
adb shell chmod +x /system/kaios/metricsd
adb push ./config.json /system/kaios/config.json
adb shell ls -l /system/kaios
