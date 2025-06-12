# Running Migrator CLI

## 基本これ

### /migrationで

migrationファイル作成

DATABASE_URL="sqlite:../bot_data.db?mode=rwc" cargo run -- generate {MIGRATION_NAME}

migration実行

<!-- DATABASE_URL="sqlite:../bot_data.db?mode=rwc" cargo run -- up -->

DATABASE_URL="sqlite:../bot_data.db?mode=rwc" cargo run -- fresh

### /で 

entity自動生成

DATABASE_URL="sqlite:bot_data.db?mode=rwc" sea-orm-cli generate entity -o src/entities

## default

- Generate a new migration file
    ```sh
    cargo run -- generate MIGRATION_NAME
    ```
- Apply all pending migrations
    ```sh
    cargo run
    ```
    ```sh
    cargo run -- up
    ```
- Apply first 10 pending migrations
    ```sh
    cargo run -- up -n 10
    ```
- Rollback last applied migrations
    ```sh
    cargo run -- down
    ```
- Rollback last 10 applied migrations
    ```sh
    cargo run -- down -n 10
    ```
- Drop all tables from the database, then reapply all migrations
    ```sh
    cargo run -- fresh
    ```
- Rollback all applied migrations, then reapply all migrations
    ```sh
    cargo run -- refresh
    ```
- Rollback all applied migrations
    ```sh
    cargo run -- reset
    ```
- Check the status of all migrations
    ```sh
    cargo run -- status
    ```
