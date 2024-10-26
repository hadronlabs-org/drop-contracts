FROM arm64v8/golang:1.22-bullseye AS go-builder

# Install minimum necessary dependencies, build Cosmos SDK, remove packages
RUN apt update
RUN apt install -y curl git build-essential


WORKDIR /code
COPY . /code/
RUN go mod edit -replace=github.com/cosmos/ibc-go/v8@v8.2.0=github.com/ratik/ibc-go/v8@v8.2.0-debug
RUN go mod tidy
RUN LEDGER_ENABLED=false make build

RUN cp /go/pkg/mod/github.com/initia\-labs/movevm@v*/api/libmovevm.`uname -m`.so /lib/libmovevm.so
RUN cp /go/pkg/mod/github.com/initia\-labs/movevm@v*/api/libcompiler.`uname -m`.so /lib/libcompiler.so

FROM arm64v8/ubuntu:20.04

WORKDIR /root

COPY --from=go-builder /code/build/initiad /usr/local/bin/initiad
COPY --from=go-builder /lib/libmovevm.so /lib/libmovevm.so
COPY --from=go-builder /lib/libcompiler.so /lib/libcompiler.so

RUN cp /lib/libmovevm.so /lib/libmovevm.`uname -m`.so 
RUN cp /lib/libcompiler.so /lib/libcompiler.`uname -m`.so 

EXPOSE 1317
EXPOSE 9090
EXPOSE 26656
EXPOSE 26657

CMD ["initiad"]