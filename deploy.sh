#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

if [ -z "$TARGET_HOST" ]
then
    echo "Please set the TARGET_HOST environment variable";
    exit 1;
fi

if [ -z "$TARGET_PATH" ]
then
    echo "Please set the $TARGET_PATH environment variable";
    exit 1;
fi

readonly TARGET_ARCH=armv7-unknown-linux-musleabihf
readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/segelflug-classifieds

set -o xtrace

cross build --release --target=${TARGET_ARCH}
rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} sudo systemctl restart segelflug-classifieds-bot

if [ -n "$SHOW_LOGS" ]
then
    ssh -t ${TARGET_HOST} journalctl -u segelflug-classifieds-bot -f
fi
