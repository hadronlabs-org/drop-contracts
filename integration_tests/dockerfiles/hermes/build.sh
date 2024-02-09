#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
docker build . -t hermes-test:$VERSION

