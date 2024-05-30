FROM golang:1.21-bullseye as builder
RUN apt update && apt install git sed -y
WORKDIR /app
RUN git clone https://github.com/hadronlabs-org/neutron-query-relayer-cli.git .
# COPY go.mod go.sum ./
RUN go mod download
RUN export GVERSION=$(echo $(git describe --tags) | sed 's/^v//')
RUN export COMMIT=$(git log -1 --format='%H')
ENV LDFLAGS "-X github.com/hadronlabs-org/neutron-query-relayer-cli/internal/app.Version=${GVERSION} -X github.com/hadronlabs-org/neutron-query-relayer-cli/internal/app.Commit=${COMMIT}" 
# COPY . .
RUN go build -ldflags "${LDFLAGS}" -a -o build/neutron_query_relayer ./cmd/neutron_query_relayer/*.go


FROM node:21.7.1-bullseye-slim as node_builder
RUN mkdir /app/ts-client -p
COPY /ts-client/ /app/ts-client
WORKDIR /app/ts-client
RUN rm -rf node_modules
RUN yarn

RUN mkdir /app/coordinator -p
COPY /coordinator/ /app/coordinator/
WORKDIR /app/coordinator
RUN rm -rf node_modules
RUN rm -f .env
RUN yarn


FROM node:21.7.1-bullseye-slim
RUN apt update && apt install ca-certificates curl -y && apt-get clean
ADD ["https://github.com/CosmWasm/wasmvm/releases/download/v1.5.2/libwasmvm.x86_64.so","https://github.com/CosmWasm/wasmvm/releases/download/v1.5.2/libwasmvm.aarch64.so","/lib/"]

COPY --from=builder /app/build/neutron_query_relayer /usr/local/bin/

RUN mkdir /coordinator
COPY --from=node_builder /app/coordinator/ /coordinator/

ENV ICQ_RUN_COMMAND "/usr/local/bin/neutron_query_relayer run"

ENTRYPOINT ["/coordinator/run.sh"]

