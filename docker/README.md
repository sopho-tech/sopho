# Docker

## Building Docker Image

1. Go to the root of the project.
2. Run the following command:

```bash
docker build -f docker/Dockerfile . -t sopho:test
```

## Running Container

Set the env variables and execute the docker run command. For example:

```bash
docker run -d -p 8000:8000 \
  --name sopho \
  -e DATABASE_URL="postgresql://admin:password@localhost:5432/sopho" \
  -e SOPHO_FRONTEND_DIR="/app/frontend/dist/" \
  -e SOPHO_ENVIRONMENT="production" \
  -e SOPHO_COOKIE_SECURE="false" \
  -e SOPHO_ADMIN_USERNAME="admin" \
  -e SOPHO_ADMIN_PASSWORD="password" \
  -e SOPHO_ADMIN_EMAIL="admin@admin.com" \
  -e SOPHO_ADMIN_FULL_NAME="admin admin" \
  sopho:latest
```

## Local Stack

`docker-compose.override.yaml` runs Sopho with PostgreSQL and [Mailpit](https://mailpit.axllent.org), a mail server that keeps every email instead of delivering it. Start the stack from the root of the project:

```bash
docker compose -p sopho -f docker/docker-compose.yaml -f docker/docker-compose.override.yaml up -d --build
```

Sopho is at <http://localhost:8000>. The emails it sends, such as invitations and password resets, are at <http://localhost:8025>.
