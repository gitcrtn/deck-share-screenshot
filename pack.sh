#!/usr/bin/env bash
SH=$(realpath $0)
ROOTDIR=$(dirname $SH)

cd $ROOTDIR
rm -rf dist/sharess
mkdir -p dist/sharess

cargo build --release
cp target/release/sharess dist/sharess
cp create_desktop.sh dist/sharess
cp sharess.desktop dist/sharess
cp LICENSE dist/sharess

cd dist
tar cvzf sharess.tgz sharess/
rm -rf sharess/
