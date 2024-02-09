#!/bin/bash
VERSION=$(cat ./package.json | jq -r '.version')
cd dockerfiles
IMAGES=$(ls -1 | grep -v build-all.sh)
for IMAGE in $IMAGES; do
    # check if docker image is already built
    if [[ "$CI" == "true" ]]; then
        DOCKERIMAGE=neutronorg/lionco-contracts:$IMAGE
        docker pull $DOCKERIMAGE-test_$VERSION
    else
        VERSION=":$VERSION"
    fi
    if [[ "$(docker images -q $DOCKERIMAGE-test$VERSION 2> /dev/null)" == "" ]]; then
        echo "Building $DOCKERIMAGE:$VERSION"
        ./$IMAGE/build.sh
        if [[ "$CI" == "true" ]]; then
            docker push $DOCKERIMAGE-test$VERSION
        fi
    else
        echo "Image $IMAGE:$VERSION already exists"
    fi
done
