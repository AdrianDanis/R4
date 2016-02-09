#!/bin/bash

set -e

target=x86_64-r4-softfloat
# Locate our multirust path
rustc=`multirust which rustc | sed 's/\/bin\/rustc$//'`
# Location to store built libraries to
outdir="$rustc/lib/rustlib/$target/lib"
# Save the location of our patch as an absolute path
patch=`pwd`/libcore_nofp.patch

if [ "$RUST_GIT_REFERENCE" != "" ]
then
    RUST_GIT_REFERENCE="--reference $RUST_GIT_REFERENCE"
fi

# Move to a temporary directory
mkdir tmp
pushd tmp
# Grab copy of rust repo
git clone $RUST_GIT_REFERENCE https://github.com/rust-lang/rust.git
# Apply the disable_float patch
pushd rust/src
patch -p0 < $patch
popd
# Build libcore and place it where our multirust compiler will find it
mkdir -p $outdir
popd
for lib in libcore librustc_unicode liballoc libcollections
do
    rustc --target $target -Z no-landing-pads --cfg disable_float --cfg stdbuild --verbose --out-dir $outdir tmp/rust/src/$lib/lib.rs
done

rm -rf tmp
