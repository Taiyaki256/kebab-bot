use migration::{Migrator, MigratorTrait};
use poise::serenity_prelude::{
    self as serenity, ComponentInteraction, CreateInteractionResponse,
    CreateInteractionResponseMessage, EventHandler, Interaction, async_trait,
};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;
use tokio::time::{Duration, interval};

mod commands;
mod entities;
mod services;

use commands::*;
use services::*;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// ユーザーデータ構造体
pub struct Data {
    database: Arc<DatabaseConnection>,
}

// 投票処理を行う共通関数
async fn handle_vote(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    database: &Arc<DatabaseConnection>,
    action: &str,
    success_message: &str,
) -> Result<(), Error> {
    let user_id = interaction.user.id.get() as i64;

    match VoteService::update_vote(database, user_id, action.to_string()).await {
        Ok(_) => {
            let response = CreateInteractionResponseMessage::new()
                .content(success_message)
                .ephemeral(true);

            interaction
                .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                .await?;
        }
        Err(e) => {
            eprintln!("投票の保存中にエラーが発生しました: {}", e);
            let response = CreateInteractionResponseMessage::new()
                .content("投票の保存に失敗しました。")
                .ephemeral(true);

            interaction
                .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                .await?;
        }
    }
    Ok(())
}

// ボタンインタラクションを処理する関数
async fn handle_button_interaction(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    database: &Arc<DatabaseConnection>,
) -> Result<(), Error> {
    // まず日付チェックを行い、必要に応じて投票をリセット
    if let Err(e) = VoteService::check_and_reset_votes_if_new_day(database).await {
        eprintln!("日付チェック中にエラーが発生しました: {}", e);
    }

    match interaction.data.custom_id.as_str() {
        "refresh_board" => {
            // 更新ボタンが押された時の処理 - 実際に掲示板データを取得して表示
            let board_data = BoardService::get_all_board_data(database).await?;
            let content = if board_data.is_empty() {
                "まだ掲示板データがありません。".to_string()
            } else {
                let mut response = String::from("🔄 掲示板データを再読み込みしました:\n");
                for data in board_data {
                    response.push_str(&format!(
                        "• サーバーID: {} | チャンネルID: {} | メッセージID: {}\n",
                        data.server_id, data.channel_id, data.message_id
                    ));
                }
                response
            };

            let response = CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true);

            interaction
                .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                .await?;
        }
        "found" => {
            // ケバブ屋が居た時のボタン
            handle_vote(
                ctx,
                interaction,
                database,
                "found",
                "🥙 「営業してる」に投票しました！",
            )
            .await?;
        }
        "not_found" => {
            // ケバブ屋が居なかった時のボタン
            handle_vote(
                ctx,
                interaction,
                database,
                "not_found",
                "❌ 「いない」に投票しました！",
            )
            .await?;
        }
        "sold_out" => {
            // 売り切れ、おしまいだった時のボタン
            handle_vote(
                ctx,
                interaction,
                database,
                "sold_out",
                "🚫 「売り切れた」に投票しました！",
            )
            .await?;
        }
        _ => {
            // 未知のボタンID
            let response = CreateInteractionResponseMessage::new()
                .content("不明なボタンです。")
                .ephemeral(true);

            interaction
                .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                .await?;
        }
    }
    Ok(())
}

// 定期的に投票期間チェックを行うバックグラウンドタスク
async fn periodic_date_check(database: Arc<DatabaseConnection>) {
    // 毎時0分に実行するため、現在時刻から次の0分までの時間を計算
    let mut interval = interval(Duration::from_secs(3600)); // 1時間ごと

    loop {
        interval.tick().await;

        match VoteService::check_and_reset_votes_if_new_day(&database).await {
            Ok(reset) => {
                if reset {
                    println!("🔄 定期チェック: 投票期間変更による投票リセットが完了しました！");
                }
            }
            Err(e) => {
                eprintln!("⚠️ 定期投票期間チェック中にエラーが発生しました: {}", e);
            }
        }
    }
}

// イベントハンドラー構造体
struct Handler {
    database: Arc<DatabaseConnection>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: serenity::Context, interaction: Interaction) {
        if let Interaction::Component(component_interaction) = interaction {
            if let Err(e) =
                handle_button_interaction(&ctx, &component_interaction, &self.database).await
            {
                eprintln!(
                    "ボタンインタラクションの処理中にエラーが発生しました: {}",
                    e
                );
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // 環境変数を読み込み
    dotenvy::dotenv().ok();

    // データベース接続
    let database = Database::connect("sqlite:bot_data.db?mode=rwc")
        .await
        .expect("データベースに接続できませんでした");

    // マイグレーションを実行
    Migrator::up(&database, None)
        .await
        .expect("マイグレーションの実行に失敗しました");

    println!("データベースの初期化が完了しました！");

    // 投票期間が変わっていたら投票をリセット
    match VoteService::check_and_reset_votes_if_new_day(&database).await {
        Ok(reset) => {
            if reset {
                println!("✅ 投票期間変更による投票リセットが完了しました！");
            } else {
                println!("ℹ️ 現在の投票期間（午後期間）は継続中です");
            }
        }
        Err(e) => {
            eprintln!("⚠️ 投票期間チェック中にエラーが発生しました: {}", e);
        }
    }

    println!("Botを起動しています...");
    println!(
        "DISCORD_TOKEN環境変数: {}",
        if std::env::var("DISCORD_TOKEN").is_ok() {
            "設定済み"
        } else {
            "未設定"
        }
    );

    // データベースをArcで包む
    let database = Arc::new(database);
    let database_for_setup = Arc::clone(&database);
    let database_for_handler = Arc::clone(&database);
    let database_for_periodic = Arc::clone(&database);

    // 定期的な日付チェックタスクを開始
    tokio::spawn(periodic_date_check(database_for_periodic));
    println!("🕒 定期日付チェックタスクを開始しました（1時間ごと）");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                help(),
                ping(),
                userinfo(),
                serverinfo(),
                create_board(),
                update_board(),
                reset_votes(),
                vote_results(),
                vote_chart(),
                vote_date_info(),
                vote_sample(),
            ],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("登録するコマンド数: {}", framework.options().commands.len());
                for command in &framework.options().commands {
                    println!("コマンド名: {}", command.name);
                }

                match poise::builtins::register_globally(ctx, &framework.options().commands).await {
                    Ok(_) => println!("✅ スラッシュコマンドの登録が完了しました！"),
                    Err(e) => eprintln!("❌ スラッシュコマンドの登録に失敗しました: {}", e),
                }

                Ok(Data {
                    database: database_for_setup,
                })
            })
        })
        .build();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    // イベントハンドラーを作成
    let handler = Handler {
        database: database_for_handler,
    };

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(handler)
        .await;
    client.unwrap().start().await.unwrap();
}
