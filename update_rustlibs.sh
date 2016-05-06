#!/bin/bash

set -e

target=x86_64-unknown-r4-nofloat
target_path=`pwd`

mkdir tmp
pushd tmp

wget -q https://raw.githubusercontent.com/phil-opp/nightly-libcore/master/install-libcore.sh
RUST_TARGET_PATH="$target_path" bash install-libcore.sh $target disable_float

wget -q https://raw.githubusercontent.com/phil-opp/nightly-libcollections/master/install-libcollections.sh
RUST_TARGET_PATH="$target_path" sh install-libcollections.sh $target

popd

rmdir tmp
