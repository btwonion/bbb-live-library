# Stage 1: Build frontend
FROM node:22-slim AS frontend-build
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build backend
FROM rust:1.83-slim AS backend-build
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app/backend

# Cache dependencies
COPY backend/Cargo.toml backend/Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

# Build real source
COPY backend/src/ src/
COPY backend/migrations/ migrations/
RUN touch src/main.rs && cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ffmpeg ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend-build /app/backend/target/release/bbb-live-library /app/bbb-live-library
COPY --from=frontend-build /app/frontend/dist /app/frontend/dist
COPY config.docker.toml /app/config.toml

RUN mkdir -p /data/recordings/thumbs

EXPOSE 8080
VOLUME ["/data"]

CMD ["/app/bbb-live-library"]
