# Sopho Backend

## Development

1. Install sea-orm-cli

```bash
cargo install sea-orm-cli
```

2. Generate Entities

```bash
sea-orm-cli generate entity --ignore-tables seaql_migrations,scheduler_job,scheduler_notification,scheduler_notification_state -u "<DB_URL>" -o src/entity
```

## Serving Frontend

1. The location `FRONTEND_DIR` in the `.env` file should point to the directory containing the frontend build.
2. The frontend build should be in the `dist` directory.
3. The backend will serve the frontend build at `/`.
