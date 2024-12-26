# build stage
FROM node:20-alpine as base

LABEL web.maintainer=alex.li@oyiyio.com \
  web.name=fragrans-storage \
  web.version=0.0.9

FROM base as build-stage

WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN npm config set registry https://registry.npmmirror.com
RUN npm install -g pnpm
RUN pnpm config set registry https://registry.npmmirror.com
RUN pnpm install --verbose
COPY . .
RUN pnpm build:prod

# production stage
FROM nginx as production-stage
RUN mkdir /app
COPY --from=build-stage /app/dist /app
COPY deploy/nginx/nginx.conf /etc/nginx/nginx.conf