# build stage
FROM node:24-alpine AS base

LABEL web.maintainer=alex.li@oyiyio.com \
  web.name=fragrans-drive \
  web.version=0.3.0

FROM base AS build-stage

WORKDIR /app
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY . .

ARG VERSION
ARG GIT_REVISION
ENV VITE_APP_VERSION=${VERSION} \
    VITE_GIT_REVISION=${GIT_REVISION}

RUN pnpm --filter @fragrans/web build:prod

# production stage
FROM nginx:alpine AS production-stage
RUN mkdir /app
COPY --from=build-stage /app/apps/web/dist /app
COPY apps/web/deploy/nginx/nginx.conf /etc/nginx/nginx.conf
COPY apps/web/deploy/nginx/default.conf.template /etc/nginx/templates/default.conf.template
ENV API_UPSTREAM=http://host.docker.internal:3821 \
    NGINX_ENVSUBST_FILTER=API_UPSTREAM
