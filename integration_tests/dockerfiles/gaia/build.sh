#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
git clone https://github.com/cosmos/gaia.git -b v14.1.0
cp ./Dockerfile ./gaia
docker build gaia -t gaia-test:$VERSION
rm -rf ./gaia