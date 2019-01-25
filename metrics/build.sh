#!/bin/bash

set -e

if [ "$(uname -s)" == "Darwin" ]; then
    HOST_ARCH_S=darwin-x86
else
    HOST_ARCH_S=linux-x86
fi

# Check that the GONK_DIR environment variable is set
# and build the .cargo/config file from it.
if [ -z ${GONK_DIR+x} ];
then
    echo "Please set GONK_DIR to the root of your Gonk directory first.";
    exit 1;
else
    # Get the product name from .config if it does exist.
    [ -f $GONK_DIR/.config ] && source $GONK_DIR/.config
    CARGO_CONFIG=`pwd`/.cargo/config
    echo "Using '$GONK_DIR' to create '$CARGO_CONFIG' for '$PRODUCT_NAME'";
    mkdir -p `pwd`/.cargo
    cat << EOF > $CARGO_CONFIG
[target.armv7-linux-androideabi]
linker="$GONK_DIR/prebuilts/gcc/$HOST_ARCH_S/arm/arm-linux-androideabi-4.9/bin/arm-linux-androideabi-gcc"
rustflags = [
  "-C", "link-arg=--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/",
]
EOF
fi

STRIP=
OPT=
TARGET=debug
while [[ $# -gt 0 ]]; do
    case "$1" in
        --release)
            OPT=--release
            TARGET=release
            ;;
        --strip)
            STRIP=yes
            ;;
    esac
    shift
done

# Needed for cross-compiling C dependencies properly.
export CFLAGS="--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/ -I$GONK_DIR/prebuilts/ndk/9/platforms/android-21/arch-arm/usr/include"

cargo build --target=armv7-linux-androideabi ${OPT}

if [ "${STRIP}" = "yes" ];
then
    # Explicitely strip the binary since even release builds have symbols.
    DAEMON=./target/armv7-linux-androideabi/release/metrics_daemon
    $GONK_DIR/prebuilts/gcc/$HOST_ARCH_S/arm/arm-linux-androideabi-4.9/bin/arm-linux-androideabi-strip $DAEMON
fi
