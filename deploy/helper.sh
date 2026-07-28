#!/usr/bin/env bash

set -euo pipefail

# 定义变量
REGISTRY_NAME="hub.docker.com"
CONTAINER_NAME="fragrans-drive"
CONTAINER_PORT=8061
CONTAINER_INNER_PORT=80
COMMIT_SHA=$(git rev-parse --short HEAD)
API_UPSTREAM=${API_UPSTREAM:-http://host.docker.internal:3821}

# 构建 fragrans-drive 镜像
docker build --pull --no-cache -t "$REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA" .

# 登录 Docker Registry
# echo $DOCKER_REGISTRY_PASSWORD | docker login $REGISTRY_NAME -u $DOCKER_REGISTRY_USER --password-stdin

# 推送 fragrans-drive 到镜像仓库
# docker push $REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA

# 从镜像仓库再次拉取 fragrans-drive 镜像
# docker pull $REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA

# 登出 Docker Registry
# docker logout

# 删除已经生成或正在运行的容器
cid=$(docker ps -aq --filter "name=^/${CONTAINER_NAME}$")

if [ -n "$cid" ]; then
  docker rm -f "$cid"
fi

# 启动服务
# Mac OS X 操作系统
if [ "$(uname -s)" = "Darwin" ]; then
  docker run --name "$CONTAINER_NAME" \
    -d \
    -p "$CONTAINER_PORT:$CONTAINER_INNER_PORT" \
    -e "API_UPSTREAM=$API_UPSTREAM" \
    --add-host host.docker.internal:host-gateway \
    --restart=always \
    "$REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA"
# GNU/Linux操作系统
elif [ "$(uname -s)" = "Linux" ]; then
  docker run --name "$CONTAINER_NAME" \
    -d \
    -p "$CONTAINER_PORT:$CONTAINER_INNER_PORT" \
    -e "API_UPSTREAM=$API_UPSTREAM" \
    --add-host host.docker.internal:host-gateway \
    --restart=always \
    -v /etc/localtime:/etc/localtime:ro \
    -v /etc/timezone:/etc/timezone \
    "$REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA"
else
  echo "Unsupported operating system: $(uname -s)" >&2
  exit 1
fi
