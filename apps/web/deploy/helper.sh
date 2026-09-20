#!/usr/bin/env bash

set -euo pipefail

# Define variables
REGISTRY_NAME="hub.docker.com"
CONTAINER_NAME="fragrans-drive"
CONTAINER_PORT=8061
CONTAINER_INNER_PORT=80
COMMIT_SHA=$(git rev-parse --short HEAD)
API_UPSTREAM=${API_UPSTREAM:-http://host.docker.internal:3821}
TRUSTED_INGRESS_CIDR=${TRUSTED_INGRESS_CIDR:-127.0.0.1/32}

# Build fragrans-drive image
docker build --pull --no-cache -t "$REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA" .

# Login to Docker Registry
# echo $DOCKER_REGISTRY_PASSWORD | docker login $REGISTRY_NAME -u $DOCKER_REGISTRY_USER --password-stdin

# Push fragrans-drive to image registry
# docker push $REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA

# Pull fragrans-drive image from registry
# docker pull $REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA

# Logout from Docker Registry
# docker logout

# Remove existing or running container
cid=$(docker ps -aq --filter "name=^/${CONTAINER_NAME}$")

if [ -n "$cid" ]; then
  docker rm -f "$cid"
fi

# Start service
# macOS operating system
if [ "$(uname -s)" = "Darwin" ]; then
  docker run --name "$CONTAINER_NAME" \
    -d \
    -p "$CONTAINER_PORT:$CONTAINER_INNER_PORT" \
    -e "API_UPSTREAM=$API_UPSTREAM" \
    -e "TRUSTED_INGRESS_CIDR=$TRUSTED_INGRESS_CIDR" \
    --add-host host.docker.internal:host-gateway \
    --restart=always \
    "$REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA"
# GNU/Linux operating system
elif [ "$(uname -s)" = "Linux" ]; then
  docker run --name "$CONTAINER_NAME" \
    -d \
    -p "$CONTAINER_PORT:$CONTAINER_INNER_PORT" \
    -e "API_UPSTREAM=$API_UPSTREAM" \
    -e "TRUSTED_INGRESS_CIDR=$TRUSTED_INGRESS_CIDR" \
    --add-host host.docker.internal:host-gateway \
    --restart=always \
    -v /etc/localtime:/etc/localtime:ro \
    -v /etc/timezone:/etc/timezone \
    "$REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA"
else
  echo "Unsupported operating system: $(uname -s)" >&2
  exit 1
fi
