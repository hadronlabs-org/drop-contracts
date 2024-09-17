#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
# git clone https://github.com/initia-labs/initia -b v0.2.15
cp ./Dockerfile ./initia
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    echo ""
    VERSION=":$VERSION"
    # new_replace="github.com/cosmos/ibc-go/v4 v4.4.2 => github.com/ratik/ibc-go/v4 v4.4.3-0.20231115171220-5c22b66cfa8c"
    # gomod_file="gaia/go.mod"
    # cp "$gomod_file" "$gomod_file.bak"
    # awk -v new_replace="$new_replace" '
    # BEGIN { replace_block=0; added=0 }
    # /replace[[:space:]]*\(/ { replace_block=1 }
    # /^[[:space:]]*\)/ { if(replace_block) { print new_replace; added=1; replace_block=0 } }
    # { print }
    # END { if(!added) { print "replace ("; print new_replace; print ")" } }
    # ' "$gomod_file.bak" > "$gomod_file"
    # cd initia
    # go mod tidy
    # cd ..
fi
docker build initia -t ${ORG}initia-test${VERSION}
# rm -rf ./initia