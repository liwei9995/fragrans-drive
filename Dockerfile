# build stage
FROM node:24-alpine AS base

LABEL web.maintainer=alex.li@oyiyio.com \
  web.name=fragrans-drive \
  web.version=0.2.0

FROM base AS build-stage

WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN npm config set registry https://registry.npmmirror.com
RUN npm install -g pnpm
RUN pnpm config set registry https://registry.npmmirror.com
RUN pnpm install --verbose
COPY . .
RUN pnpm build:prod

# production stage
FROM nginx AS production-stage
RUN mkdir /app
COPY --from=build-stage /app/dist /app
COPY deploy/nginx/nginx.conf /etc/nginx/nginx.conf