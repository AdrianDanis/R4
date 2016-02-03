#!/bin/bash

set -e

target=x86_64-r4-softfloat
# Locate our multirust path
rustc=`multirust which rustc | sed 's/\/bin\/rustc$//'`
# Location to store built libraries to
outdir="$rustc/lib/rustlib/$target/lib"
# Save the location of our patch as an absolute path
patch=`pwd`/libcore_nofp.patch

# Move to a temporary directory
mkdir tmp
pushd tmp
# Grab copy of rust repo
git clone https://github.com/rust-lang/rust.git
# Apply the disable_float patch
pushd rust/src
patch -p0 < $patch
popd
# Build libcore and place it where our multirust compiler will find it
mkdir -p $outdir
popd
rustc --target $target --cfg disable_float -Z no-landing-pads --out-dir $outdir tmp/rust/src/libcore/lib.rs

rm -rf tmp
