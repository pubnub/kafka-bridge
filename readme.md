# PubNub NATs Bridge

Bring push notifications and streaming events to Mobile and Web clients
based on a NATs channels.
Easy drop-in sidecar.
Dashboard management page included.

The application is written in Rust.
The application will listen on NATS channels specified by runtime
and deploytime configuration.
Built-in security model allows encyrpted data in motion using 2048bit TLS.

## Deploying Sidecar in Kubernetes (K8s)

...
Add Secrets to secret store.
Add YAML sidecar

## Deploying Docker Compose

`docker run -p 4444:4444 nats -p 4444`

...

## Reference Links

https://hub.docker.com/_/nats
