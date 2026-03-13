# build stage
FROM node:25-alpine as base

LABEL svc.maintainer=alex.li@oyiyio.com \
      svc.name=yi-svc-storage \
      svc.version=0.0.15

FROM base as build-stage

RUN npm i -g pnpm

# Run as an unprivileged user.
RUN addgroup -S oyiyio && adduser -S -G oyiyio oyiyio
RUN mkdir /app && chown oyiyio /app
USER oyiyio

WORKDIR /app
COPY --chown=oyiyio:oyiyio package.json pnpm-lock.yaml /app/
RUN pnpm install --frozen-lockfile
COPY --chown=oyiyio:oyiyio . .
ENV NODE_ENV=production
RUN pnpm run build

# production stage
FROM base as production-stage
RUN npm i -g pnpm
RUN mkdir /app
RUN mkdir -p /app/bucket

WORKDIR /app
COPY --from=build-stage /app/package.json /app/pnpm-lock.yaml ./
COPY --from=build-stage /app/config ./config
COPY --from=build-stage /app/dist ./dist
ENV NODE_ENV=production
RUN pnpm install --prod --frozen-lockfile

# Start the server using the production build
CMD [ "node", "dist/main.js" ]

EXPOSE 3847