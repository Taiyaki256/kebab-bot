# --- ビルドステージ (変更なし) ---
FROM rust:1.87-slim as builder
WORKDIR /app
RUN apt-get update && apt-get install -y \
    pkg-config \
    libfreetype6-dev \
    libfontconfig1-dev \
    build-essential \
    cmake \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/
COPY src ./src
COPY migration/src ./migration/src
RUN cargo build --release

# --- ランタイムステージ (推奨される修正) ---
FROM debian:bookworm-slim

# ランタイム依存関係のインストール
RUN apt-get update && apt-get install -y \
    libfreetype6 \
    libfontconfig1 \
    fonts-noto-cjk \
    fonts-liberation \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 非rootユーザーとグループを作成
# ★変更点1: --create-home を追加して、ユーザーのホームディレクトリを作成
RUN groupadd --system --gid 1001 appgroup && \
    useradd --system --uid 1001 --gid appgroup --create-home --shell /bin/false appuser

# アプリケーションバイナリをコピー
COPY --from=builder /app/target/release/kebab-bot /usr/local/bin/kebab-bot

# バイナリの所有権と実行権限をrootで設定
RUN chown appuser:appgroup /usr/local/bin/kebab-bot && \
    chmod +x /usr/local/bin/kebab-bot

# ユーザーを非rootに切り替え
USER appuser

# ★変更点2: ワーキングディレクトリをユーザーのホームディレクトリに設定
# このディレクトリは appuser が所有しているので、ファイルの書き込みが可能
WORKDIR /home/appuser

# アプリケーションの実行
CMD ["kebab-bot"]