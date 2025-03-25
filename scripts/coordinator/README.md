## Coordinator start

In order to have ability to automatically restart coordinator service on blockchain node restart use command like this:

`./start.sh <PATH TO ENV CONFIGURATION FILE>`

## Docker build

To build docker image run command:

`./build.sh`

## Docker run

To run docker image run command:

`docker run -it --rm --restart=always --name coordinator --env-file <PATH TO ENV CONFIGURATION FILE> dropprotocol/coordinator`

## Deploy docker image

To deploy docker image run command:

```
docker tag dropprotocol/coordinator dropprotocol/coordinator:<VERSION>
docker push dropprotocol/coordinator:<VERSION>
```
