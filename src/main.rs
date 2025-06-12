use chrono::Datelike;
use migration::{Migrator, MigratorTrait};
use poise::{
    CreateReply,
    serenity_prelude::{
        self as serenity, ButtonStyle, Colour, ComponentInteraction, CreateActionRow, CreateButton,
        CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, EditMessage,
        EventHandler, Interaction, async_trait,
    },
};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

mod entities;
mod services;

use services::*;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// ユーザーデータ構造体
pub struct Data {
    database: Arc<DatabaseConnection>,
}

/// ping コマンド
#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// ユーザー情報を表示するコマンド
#[poise::command(slash_command)]
async fn userinfo(
    ctx: Context<'_>,
    #[description = "ユーザーを選択してください"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!(
        "**{}** のユーザー情報:\n\
        ユーザーID: {}\n\
        アカウント作成日: <t:{}:F>\n\
        Bot: {}",
        u.name,
        u.id,
        u.created_at().timestamp(),
        if u.bot { "Yes" } else { "No" }
    );
    ctx.say(response).await?;
    Ok(())
}

/// サーバー情報を表示するコマンド
#[poise::command(slash_command)]
async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
    let (guild_id, guild_name, member_count, channel_count) = {
        let guild = ctx.guild().unwrap();
        (
            guild.id,
            guild.name.clone(),
            guild.member_count,
            guild.channels.len(),
        )
    };
    let created_timestamp = guild_id.created_at().timestamp();

    let response = format!(
        "**{}** のサーバー情報:\n\
        サーバーID: {}\n\
        メンバー数: {}\n\
        チャンネル数: {}\n\
        作成日: <t:{}:F>",
        guild_name, guild_id, member_count, channel_count, created_timestamp
    );
    ctx.say(response).await?;
    Ok(())
}

/// help コマンド
#[poise::command(prefix_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made with poise! Visit https://github.com/serenity-rs/poise",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

// 板を出すコマンド
#[poise::command(slash_command)]
async fn create_board(ctx: Context<'_>) -> Result<(), Error> {
    let res = ctx.say("板").await?;

    let server_id = ctx.guild_id().unwrap().get() as i64;
    let channel_id = ctx.channel_id().get() as i64;
    let message_id = res.message().await?.id.get() as i64;

    BoardService::update_board_data(
        &ctx.data().database,
        server_id,
        Some(channel_id),
        Some(message_id),
    )
    .await?;

    let rep = ctx
        .reply_builder(CreateReply::default())
        .content(format!(
            "掲示板データを保存しました。\nサーバーID: {}\nチャンネルID: {}\nメッセージID: {}",
            server_id, channel_id, message_id
        ))
        .ephemeral(true);
    ctx.send(rep).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn update_board(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let board_data = BoardService::get_all_board_data(&ctx.data().database).await?;
    if board_data.is_empty() {
        let button = CreateButton::new("refresh_board")
            .label("更新")
            .style(ButtonStyle::Primary);

        let action_row = CreateActionRow::Buttons(vec![button]);

        let rep = ctx
            .reply_builder(CreateReply::default())
            .content("保存された掲示板データはありません。")
            .components(vec![action_row])
            .ephemeral(true);
        ctx.send(rep).await?;
        return Ok(());
    }

    let mut response = String::from("保存された掲示板データ:\n");
    for data in board_data {
        // Discord上のメッセージを編集する例
        if let Ok(channel) = ctx
            .serenity_context()
            .http
            .get_channel(serenity::ChannelId::new(data.channel_id as u64))
            .await
        {
            if let serenity::Channel::Guild(channel) = channel {
                if let Ok(mut message) = channel
                    .message(&ctx.serenity_context().http, data.message_id as u64)
                    .await
                {
                    // ここでメッセージ内容を編集
                    let now = chrono::Utc::now();
                    let date = now.date_naive();
                    let date_str = date.format("%m/%d").to_string();
                    // 曜日
                    let weekday = now.weekday();
                    let weekday_str = match weekday {
                        chrono::Weekday::Mon => "月",
                        chrono::Weekday::Tue => "火",
                        chrono::Weekday::Wed => "水",
                        chrono::Weekday::Thu => "木",
                        chrono::Weekday::Fri => "金",
                        chrono::Weekday::Sat => "土",
                        chrono::Weekday::Sun => "日",
                    };
                    let found_button = CreateButton::new("found")
                        .label("営業してる")
                        .style(ButtonStyle::Primary);
                    let not_found_button = CreateButton::new("not_found")
                        .label("いない")
                        .style(ButtonStyle::Secondary);
                    let sold_out_button = CreateButton::new("sold_out")
                        .label("売り切れた")
                        .style(ButtonStyle::Danger);
                    let action_row = CreateActionRow::Buttons(vec![
                        found_button,
                        not_found_button,
                        sold_out_button,
                    ]);

                    // 投票結果を取得
                    let found_count = VoteService::count_votes_by_action(
                        &ctx.data().database,
                        "found".to_string(),
                    )
                    .await
                    .unwrap_or(0);
                    let not_found_count = VoteService::count_votes_by_action(
                        &ctx.data().database,
                        "not_found".to_string(),
                    )
                    .await
                    .unwrap_or(0);
                    let sold_out_count = VoteService::count_votes_by_action(
                        &ctx.data().database,
                        "sold_out".to_string(),
                    )
                    .await
                    .unwrap_or(0);

                    let embed = CreateEmbed::new()
                        .title(format!("{}({})のケバブ情報掲示板", date_str, weekday_str))
                        .description(format!(
                            "**📊 投票結果**\n\
                            🥙 営業してる: {}票\n\
                            ❌ いない: {}票\n\
                            🚫 売り切れた: {}票\n\n\
                            更新日時: <t:{}:F>",
                            found_count,
                            not_found_count,
                            sold_out_count,
                            now.timestamp()
                        ))
                        .colour(Colour::from_rgb(0, 255, 0))
                        .timestamp(now);
                    let msg = EditMessage::new().embed(embed).components(vec![action_row]);
                    let _ = message.edit(&ctx.serenity_context().http, msg).await;
                    response.push_str(&format!(
                        "メッセージID: {} を編集しました。\n",
                        data.message_id
                    ));
                } else {
                    response.push_str(&format!(
                        "メッセージID: {} の取得に失敗しました。\n",
                        data.message_id
                    ));
                }
            }
        }
    }

    let rep = ctx
        .reply_builder(CreateReply::default())
        .content(response)
        .ephemeral(true);
    ctx.send(rep).await?;
    Ok(())
}

/// 投票をリセットするコマンド
#[poise::command(slash_command)]
async fn reset_votes(ctx: Context<'_>) -> Result<(), Error> {
    match VoteService::delete_all_vote(&ctx.data().database).await {
        Ok(result) => {
            let rep = ctx
                .reply_builder(CreateReply::default())
                .content(format!(
                    "✅ {}件の投票をリセットしました。",
                    result.rows_affected
                ))
                .ephemeral(true);
            ctx.send(rep).await?;
        }
        Err(e) => {
            eprintln!("投票のリセット中にエラーが発生しました: {}", e);
            let rep = ctx
                .reply_builder(CreateReply::default())
                .content("❌ 投票のリセットに失敗しました。")
                .ephemeral(true);
            ctx.send(rep).await?;
        }
    }
    Ok(())
}

/// 投票結果を確認するコマンド
#[poise::command(slash_command)]
async fn vote_results(ctx: Context<'_>) -> Result<(), Error> {
    let found_count = VoteService::count_votes_by_action(&ctx.data().database, "found".to_string())
        .await
        .unwrap_or(0);
    let not_found_count =
        VoteService::count_votes_by_action(&ctx.data().database, "not_found".to_string())
            .await
            .unwrap_or(0);
    let sold_out_count =
        VoteService::count_votes_by_action(&ctx.data().database, "sold_out".to_string())
            .await
            .unwrap_or(0);

    let embed = CreateEmbed::new()
        .title("📊 現在の投票結果")
        .description(format!(
            "🥙 営業してる: {}票\n\
            ❌ いない: {}票\n\
            🚫 売り切れた: {}票\n\n\
            合計: {}票",
            found_count,
            not_found_count,
            sold_out_count,
            found_count + not_found_count + sold_out_count
        ))
        .colour(Colour::from_rgb(52, 152, 219))
        .timestamp(chrono::Utc::now());

    let rep = ctx.reply_builder(CreateReply::default()).embed(embed);
    ctx.send(rep).await?;
    Ok(())
}

/// 投票結果のグラフを生成するコマンド
#[poise::command(slash_command)]
async fn vote_chart(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    // 投票データを取得
    let votes = VoteService::get_all_votes(&ctx.data().database).await?;

    if votes.is_empty() {
        ctx.say("📊 まだ投票データがありません。").await?;
        return Ok(());
    }

    // 現在の投票数を取得
    let found_count = VoteService::count_votes_by_action(&ctx.data().database, "found".to_string())
        .await
        .unwrap_or(0);
    let not_found_count =
        VoteService::count_votes_by_action(&ctx.data().database, "not_found".to_string())
            .await
            .unwrap_or(0);
    let sold_out_count =
        VoteService::count_votes_by_action(&ctx.data().database, "sold_out".to_string())
            .await
            .unwrap_or(0);

    // 時系列グラフを生成
    let timeline_path = "vote_timeline.png";
    match ChartService::generate_vote_timeline_chart(votes, timeline_path).await {
        Ok(_) => {
            // ファイルを送信
            let file = serenity::CreateAttachment::path(timeline_path).await?;
            let rep = ctx
                .reply_builder(CreateReply::default())
                .content("📈 **投票の時系列グラフ**")
                .attachment(file);
            ctx.send(rep).await?;
        }
        Err(e) => {
            eprintln!("グラフ生成エラー: {}", e);
            ctx.say("❌ グラフの生成に失敗しました。").await?;
        }
    }

    // 円グラフを生成
    let pie_path = "vote_pie.png";
    match ChartService::generate_vote_pie_chart(
        found_count,
        not_found_count,
        sold_out_count,
        pie_path,
    )
    .await
    {
        Ok(_) => {
            // ファイルを送信
            let file = serenity::CreateAttachment::path(pie_path).await?;
            let rep = ctx
                .reply_builder(CreateReply::default())
                .content("🥧 **現在の投票結果（円グラフ）**")
                .attachment(file);
            ctx.send(rep).await?;
        }
        Err(e) => {
            eprintln!("円グラフ生成エラー: {}", e);
            ctx.say("❌ 円グラフの生成に失敗しました。").await?;
        }
    }

    Ok(())
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
