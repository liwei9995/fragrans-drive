# build stage
FROM node:18-alpine as build-stage

LABEL web.maintainer="alex.li@oyiyio.com" \
  web.name="fragrans-storage" \
  web.version="0.0.5"

# Run as an unprivileged user.
RUN addgroup -S oyiyio && adduser -S -G oyiyio oyiyio
RUN mkdir /app && chown oyiyio /app
USER oyiyio

WORKDIR /app
COPY --chown=oyiyio:oyiyio package*.json ./
RUN npm install --verbose
COPY --chown=oyiyio:oyiyio . .
RUN npm run build

# production stage
FROM nginx as production-stage
RUN mkdir /app
COPY --from=build-stage /app/dist /app
COPY deploy/nginx/nginx.conf /etc/nginx/nginx.conf