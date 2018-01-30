#!/bin/bash

set -e

pushd ~/hinterland

if type ~/.cargo/bin/cargo-clippy > /dev/null; then
  echo "Using cache Rust nightly and clippy"
else
  ~/rust/bin/cargo install clippy
  PATH=$PATH:~/rust/bin
fi
popd
