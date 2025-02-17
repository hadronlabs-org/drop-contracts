ARG IMG_TAG=latest

FROM golang:1.23-bullseye AS builder
WORKDIR /src/app/
COPY go.mod go.sum* ./
RUN go mod download
COPY . .
ENV PACKAGES="curl make git libc-dev bash gcc python3"
RUN apt install -y $PACKAGES
RUN make install

FROM golang:1.23-bullseye
ARG IMG_TAG
RUN apt install -y bash curl
COPY --from=builder /go/bin/gaiad /usr/local/bin/
COPY --from=builder /go/pkg/mod/ /go/pkg/mod/
EXPOSE 26656 26657 1317 9090
USER 0

ENTRYPOINT ["gaiad"]

