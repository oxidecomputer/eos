#!/bin/bash
#:
#: name = "build-and-test"
#: variety = "basic"
#: target = "helios"
#: rust_toolchain = "stable"
#: output_rules = [
#:   "/work/debug/*",
#:   "/work/release/*",
#:   "/work/bld",
#:   "/work/build.ninja",
#: ]
#:

set -o errexit
set -o pipefail
set -o xtrace

cargo --version
rustc --version

banner "build"
cargo check
cargo build
cargo build --release

for x in debug release
do
    mkdir -p /work/$x
    cp target/$x/eos /work/$x/
done

banner "check"
cargo fmt -- --check
cargo clippy -- --deny warnings

banner "test"

eos=`pwd`/target/release/eos

pushd /tmp
rm -rf illumos-gate
git clone -b eos https://github.com/oxidecomputer/illumos-gate
cd illumos-gate
ptime -m $eos
cp build.ninja /work/
ptime -m ninja
cp -r bld /work
