# Metron Backend

## Development
1. Install sea-orm-cli
```bash
cargo install sea-orm-cli
```
2. Generate migrations
```bash
sea-orm-cli migrate generate
```
3. Run migrations
```bash
sea-orm-cli migrate run
```
4. Generate Entities
```bash
sea-orm-cli generate entity -u sqlite://./data/database.db -o src/entity
```

## Serving Frontend

1. The location `FRONTEND_DIR` in the `.env` file should point to the directory containing the frontend build.
2. The frontend build should be in the `dist` directory.
3. The backend will serve the frontend build at `/`.
