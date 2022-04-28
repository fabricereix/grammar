#!/bin/bash
set -eu
os="$1"

cargo build --release
strip target/release/grammar

package_dir="target/archive/grammar-$VERSION"
mkdir -p "$package_dir"
cp target/release/grammar "$package_dir"

upload_dir="target/upload"
mkdir -p "$upload_dir"
tarball_file="grammar-$VERSION-x86_64-$os.tar.gz"

tar cvfz "target/upload/$tarball_file" -C "$(dirname "$package_dir")" "grammar-$VERSION"