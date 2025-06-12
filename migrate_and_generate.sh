#!/bin/bash

# Migration実行とEntity生成スクリプト

echo "🔄 Starting migration and entity generation..."

# /migration ディレクトリでマイグレーション実行
echo "📁 Moving to migration directory..."
cd migration || exit 1

echo "🗄️ Running fresh migration..."
DATABASE_URL="sqlite:../bot_data.db?mode=rwc" cargo run -- fresh

if [ $? -eq 0 ]; then
    echo "✅ Migration completed successfully"
else
    echo "❌ Migration failed"
    exit 1
fi

# ルートディレクトリに戻る
echo "📁 Moving back to root directory..."
cd ..

# エンティティ自動生成
echo "🔧 Generating entities..."
DATABASE_URL="sqlite:bot_data.db?mode=rwc" sea-orm-cli generate entity -o src/entities

if [ $? -eq 0 ]; then
    echo "✅ Entity generation completed successfully"
else
    echo "❌ Entity generation failed"
    exit 1
fi

echo "🎉 All tasks completed successfully!"
