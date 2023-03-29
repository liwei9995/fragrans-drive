#!/usr/bin/bash

# 定义变量
REGISTRY_NAME="docker.oyiyio.com"
CONTAINER_NAME="yi-drive"
CONTAINER_PORT=8060
CONTAINER_INNER_PORT=80
COMMIT_SHA=$(git rev-parse --short HEAD)

if [ ! $BUILD_IMAGE_TAG ]; then
  BUILD_IMAGE_TAG=$COMMIT_SHA
fi

echo 'The docker image being deployed is: ${REGISTRY_NAME}/${CONTAINER_NAME}:${BUILD_IMAGE_TAG}.'

# 登录 Docker Registry
echo $DOCKER_REGISTRY_PASSWORD | docker login $REGISTRY_NAME -u $DOCKER_REGISTRY_USER --password-stdin

# 从镜像仓库再次拉取 yi-drive 镜像
docker pull $REGISTRY_NAME/$CONTAINER_NAME

# 登出 Docker Registry
docker logout

# 删除已经生成或正在运行的容器
cid=$(docker ps -a | grep "$CONTAINER_NAME" | awk '{print $1}')

if [ "$cid" != "" ]; then
  docker rm -f $cid
fi

# 启动服务
# Mac OS X 操作系统
if [ "$(uname)" == "Darwin" ]; then
  docker run --name $CONTAINER_NAME \
    -d \
    -p $CONTAINER_PORT:$CONTAINER_INNER_PORT \
    --restart=always \
    $REGISTRY_NAME/$CONTAINER_NAME
# GNU/Linux操作系统
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
  docker run --name $CONTAINER_NAME \
    -d \
    -p $CONTAINER_PORT:$CONTAINER_INNER_PORT \
    --restart=always \
    -v /etc/localtime:/etc/localtime:ro \
    -v /etc/timezone:/etc/timezone \
    $REGISTRY_NAME/$CONTAINER_NAME
fi
