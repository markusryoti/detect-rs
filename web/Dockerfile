FROM node:20-alpine AS builder

WORKDIR /app

COPY package.json package.json

RUN npm install --frozen-lockfile

COPY . .

ENV VITE_API_URL=http://api.detection-app.local

RUN npm run build

FROM caddy:2.7.6-alpine

WORKDIR /usr/share/caddy

RUN rm -f Caddyfile

COPY Caddyfile /etc/caddy/Caddyfile

COPY --from=builder /app/dist .

# Expose port 80, which is the default HTTP port Caddy listens on.
EXPOSE 80

# Caddy's default CMD is already set to run the server, so we don't need to specify it here.
# The Caddyfile will instruct Caddy to serve the static files.
