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
