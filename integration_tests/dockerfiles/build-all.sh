#!/bin/bash
VERSION=$(cat ./package.json | jq -r '.version')
cd dockerfiles
IMAGES=$(ls -1 | grep -v build-all.sh)
for IMAGE in $IMAGES; do
    # check if docker image is already built
    if [[ "$(docker images -q $IMAGE-test:$VERSION 2> /dev/null)" == "" ]]; then
        echo "Building $IMAGE:$VERSION"
        ./$IMAGE/build.sh
    else
        echo "Image $IMAGE:$VERSION already exists"
    fi
done
