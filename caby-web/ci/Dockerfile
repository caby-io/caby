# Build Stage
FROM node:24-bookworm-slim AS build

ENV CI=true
WORKDIR /app

RUN npm install -g pnpm

COPY . .

RUN --mount=type=cache,id=pnpm,target=/pnpm/store \
    pnpm install --frozen-lockfile
RUN pnpm build
RUN pnpm deploy --filter=caby-web --legacy --prod out

# Release Stage
FROM gcr.io/distroless/nodejs24-debian13
# FROM gcr.io/distroless/base:debug

WORKDIR /app

ENV NODE_ENV=production
# ENV ORIGIN=http://localhost:8080

COPY --from=build /app/out/ .

EXPOSE 3000
CMD ["build/index.js"]
