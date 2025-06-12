# Kebab Bot

Rust、Serenity、Poiseを使用したDiscord Botです。

## セットアップ

1. Discord Developer Portal (https://discord.com/developers/applications) でアプリケーションを作成
2. Botページでトークンを取得
3. `.env.example`を`.env`にコピーして、トークンを設定:
   ```
   cp .env.example .env
   ```
4. `.env`ファイルを編集してトークンを設定:
   ```
   DISCORD_TOKEN=your_actual_bot_token_here
   ```

## 実行

```bash
cargo run
```

```bash
cargo build --release
```


## 技術スタック

- [Rust](https://www.rust-lang.org/)
- [Serenity](https://github.com/serenity-rs/serenity) - Discord API wrapper
- [Poise](https://github.com/serenity-rs/poise) - Command framework
- [Tokio](https://tokio.rs/) - Async runtime
