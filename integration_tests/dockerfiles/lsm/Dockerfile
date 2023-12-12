ARG IMG_TAG=latest

FROM golang:1.18-alpine AS builder
WORKDIR /src/app/
COPY go.mod go.sum* ./
RUN go mod download
COPY . .
ENV PACKAGES curl make git libc-dev bash gcc linux-headers eudev-dev python3
RUN apk add --no-cache $PACKAGES
RUN CGO_ENABLED=0 make install


FROM golang:1.18-alpine
ARG IMG_TAG
RUN apk add --no-cache bash jq
COPY --from=builder /go/bin/liquidstakingd /usr/local/bin/
EXPOSE 26656 26657 1317 9090
USER 0

ENTRYPOINT ["liquidstakingd"]

