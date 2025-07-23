ARG IMG_TAG=latest

FROM golang:1.21-bullseye AS query-relayer-builder
RUN mkdir /app
WORKDIR /app
COPY ./docker_data/neutron-query-relayer-cli/go.mod ./docker_data/neutron-query-relayer-cli/go.sum ./
RUN go mod download
COPY ./docker_data/neutron-query-relayer-cli .
RUN go build -a -o build/neutron_query_relayer ./cmd/neutron_query_relayer/*.go


FROM golang:1.23-bookworm AS neutron-builder

ENV GOTOOLCHAIN=go1.23.4

# Download go dependencies
WORKDIR /neutron
COPY ./docker_data/neutron/go.mod ./docker_data/neutron/go.sum ./
RUN --mount=type=cache,target=/root/.cache/go-build \
    --mount=type=cache,target=/root/go/pkg/mod \
    go mod download

# Cosmwasm - Download correct libwasmvm version
RUN WASMVM_VERSION=$(go list -m github.com/CosmWasm/wasmvm/v2 | cut -d ' ' -f 2) && \
    wget https://github.com/CosmWasm/wasmvm/releases/download/$WASMVM_VERSION/libwasmvm_muslc.$(uname -m).a \
    -O /lib/libwasmvm_muslc.$(uname -m).a && \
    # verify checksum
    wget https://github.com/CosmWasm/wasmvm/releases/download/$WASMVM_VERSION/checksums.txt -O /tmp/checksums.txt && \
    sha256sum /lib/libwasmvm_muslc.$(uname -m).a | grep $(cat /tmp/checksums.txt | grep libwasmvm_muslc.$(uname -m) | cut -d ' ' -f 1)

# Copy the remaining files
COPY ./docker_data/neutron .

# Build neutrond binary
RUN --mount=type=cache,target=/root/.cache/go-build \
    --mount=type=cache,target=/root/go/pkg/mod \
    go build \
    -mod=readonly \
    -trimpath \
    -o /neutron/build/neutrond \
    /neutron/cmd/neutrond

FROM node:23.7-bookworm
COPY --from=query-relayer-builder /app/build/neutron_query_relayer /usr/local/bin/
COPY --from=neutron-builder /neutron/build/neutrond /usr/local/bin/

ADD ["https://github.com/CosmWasm/wasmvm/releases/download/v1.5.2/libwasmvm.x86_64.so","https://github.com/CosmWasm/wasmvm/releases/download/v1.5.2/libwasmvm.aarch64.so","/lib/"]
WORKDIR /app
COPY ./docker_data/ts-client/ ./ts-client/
WORKDIR /app/tools/coordinator/ 
COPY ./docker_data/coordinator .
RUN yarn install

RUN chmod +x ./start.sh

ENTRYPOINT ["./start.sh"]


