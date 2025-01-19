#!/usr/bin/bash

# 定义变量
REGISTRY_NAME="docker.oyiyio.com"
CONTAINER_NAME="yi-drive"
COMMIT_SHA=$(git rev-parse --short HEAD)
TAG_LATEST="latest"

echo 'The docker image that will be built with tag is: '$REGISTRY_NAME'/'$CONTAINER_NAME':'$BUILD_IMAGE_TAG''

# 构建 yi-drive 镜像
docker build --platform linux/amd64 --pull --no-cache -t $REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA -t $REGISTRY_NAME/$CONTAINER_NAME:$TAG_LATEST .

# 登录 Docker Registry
echo $DOCKER_REGISTRY_PASSWORD | docker login $REGISTRY_NAME -u $DOCKER_REGISTRY_USER --password-stdin

# 推送 yi-drive 到镜像仓库
docker push $REGISTRY_NAME/$CONTAINER_NAME:$COMMIT_SHA
docker push $REGISTRY_NAME/$CONTAINER_NAME:$TAG_LATEST

# 登出 Docker Registry
docker logout