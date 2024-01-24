#!/bin/bash
set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_ARCH=aarch64-unknown-linux-gnu
readonly LINK_FLAGS='-L /usr/arm-linux-gnueabihf/lib/ -L /usr/lib/arm-linux-gnueabihf/'

chmod -R 777 ./target/

RUSTFLAGS=${LINK_FLAGS} cross build --release --target=${TARGET_ARCH}

