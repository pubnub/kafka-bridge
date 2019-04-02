# PubNub NATS Bridge

ARG RESTY_IMAGE_BASE="alpine"
ARG RESTY_IMAGE_TAG="3.8"

FROM ${RESTY_IMAGE_BASE}:${RESTY_IMAGE_TAG}
LABEL maintainer="Stephen Blum <stephen@pubnub.com>"

RUN apk add --no-cache --virtual build-deps rust cargo

# workdir
# copy rust files
# 
# cargo build

RUN apk del build-deps

# config stuff
# run app
