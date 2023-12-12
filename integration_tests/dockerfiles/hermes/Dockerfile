FROM ubuntu:23.04
ARG HERMES_VERSION=v1.4.0
WORKDIR /app
RUN apt-get update && apt-get install -y wget && \
    PLATFORM=`uname -a | awk '{print $(NF-1)}'` && \
    VERSION=$HERMES_VERSION && \
    TARNAME="hermes-${VERSION}-${PLATFORM}-unknown-linux-gnu.tar.gz" && \
    wget "https://github.com/informalsystems/hermes/releases/download/${VERSION}/${TARNAME}" && \
    tar -xf "$TARNAME" && \
    mv ./hermes /usr/local/bin/ && \
    rm -rf "$TARNAME"

ENTRYPOINT ["hermes", "start"]