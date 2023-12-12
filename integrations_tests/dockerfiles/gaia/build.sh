#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
git clone https://github.com/cosmos/gaia.git -b v14.0.0-rc1
cp ./Dockerfile ./gaia
docker build gaia -t gaia
rm -rf ./gaia