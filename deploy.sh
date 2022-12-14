#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_HOST=pi@raspberrypi.local
readonly TARGET_PATH=/home/pi
readonly TARGET_ARCH=armv7-unknown-linux-musleabihf
readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/plug-and-play-fs
export CC=/usr/local/bin/arm-linux-musleabihf-gcc-8

cargo build --release --target=${TARGET_ARCH}
rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
rsync -r static ${TARGET_HOST}:${TARGET_PATH}
rsync -r private ${TARGET_HOST}:${TARGET_PATH}
# ssh -t ${TARGET_HOST} ${TARGET_PATH}/plug-and-play-fs