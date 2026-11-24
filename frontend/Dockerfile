# Ohngea PBX frontend — vite build served by nginx, /api proxied to api:8000.
# UI at http://localhost:${FRONTEND_PORT:-8060} (root .env); api stays on $API_PORT.

# ---------- Builder (Vite + Tailwind) ----------
ARG NPM_REGISTRY=https://registry.npmjs.org
FROM node:24-alpine AS builder
ARG NPM_REGISTRY
WORKDIR /app
COPY package*.json ./
RUN npm config set registry "$NPM_REGISTRY" && npm install --legacy-peer-deps
COPY . ./
RUN npm run build

# ---------- Runtime (nginx) ----------
FROM nginx:alpine AS runtime
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost/ > /dev/null || exit 1
