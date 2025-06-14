use crate::{Context, Error, services::*};
use chrono::Datelike;
use poise::{
    CreateReply,
    serenity_prelude::{
        ButtonStyle, ChannelId, Colour, CreateActionRow, CreateButton, CreateEmbed, EditMessage,
    },
};
use std::time::Duration;
use tokio::time::sleep;

/// 板を出すコマンド
#[poise::command(slash_command)]
pub async fn create_board(ctx: Context<'_>) -> Result<(), Error> {
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
pub async fn update_board(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // 日付チェックを行い、必要に応じて投票をリセット
    if let Err(e) = VoteService::check_and_reset_votes_if_new_day(&ctx.data().database).await {
        eprintln!("日付チェック中にエラーが発生しました: {}", e);
    }

    // 投票データを取得してタイムラインチャートを生成
    let votes = VoteService::get_all_votes(&ctx.data().database).await?;
    if !votes.is_empty() {
        let timeline_path = "vote_timeline.png";
        if let Err(e) = ChartService::generate_vote_timeline_chart(votes, timeline_path).await {
            eprintln!("タイムラインチャート生成エラー: {}", e);
        }
    }

    let board_data = BoardService::get_all_board_data(&ctx.data().database).await?;
    if board_data.is_empty() {
        handle_empty_board_data(&ctx).await?;
        return Ok(());
    }

    let response = update_all_board_messages(&ctx, board_data).await?;

    let rep = ctx
        .reply_builder(CreateReply::default())
        .content(response)
        .ephemeral(true);
    ctx.send(rep).await?;
    Ok(())
}

/// 保存された掲示板データがない場合の処理
async fn handle_empty_board_data(ctx: &Context<'_>) -> Result<(), Error> {
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
    Ok(())
}

/// 全ての掲示板メッセージを更新する
async fn update_all_board_messages(
    ctx: &Context<'_>,
    board_data: Vec<crate::entities::board_data::Model>,
) -> Result<String, Error> {
    let mut response = String::from("保存された掲示板データ:\n");

    // タイムラインチャートが存在するかチェック
    let timeline_path = "vote_timeline.png";
    let chart_exists = std::path::Path::new(timeline_path).exists();

    // embedとボタンを一度だけ作成
    let (embed, action_row) = create_board_embed_and_buttons(ctx, chart_exists).await?;

    for (index, data) in board_data.iter().enumerate() {
        // Rate limit対策: 複数メッセージがある場合は間隔を空ける
        if index > 0 {
            sleep(Duration::from_millis(500)).await;
        }

        match update_single_board_message(ctx, data, &embed, &action_row).await {
            Ok(message) => response.push_str(&message),
            Err(e) => {
                response.push_str(&format!(
                    "メッセージID: {} の更新中にエラーが発生しました: {}\n",
                    data.message_id, e
                ));
            }
        }
    }

    Ok(response)
}

/// 単一の掲示板メッセージを更新する
async fn update_single_board_message(
    ctx: &Context<'_>,
    data: &crate::entities::board_data::Model,
    embed: &CreateEmbed,
    action_row: &CreateActionRow,
) -> Result<String, Error> {
    let channel = ctx
        .serenity_context()
        .http
        .get_channel(ChannelId::new(data.channel_id as u64))
        .await?;

    if let poise::serenity_prelude::Channel::Guild(channel) = channel {
        let mut message = channel
            .message(&ctx.serenity_context().http, data.message_id as u64)
            .await?;

        let msg = EditMessage::new()
            .content("")
            .embed(embed.clone())
            .components(vec![action_row.clone()]);

        message.edit(&ctx.serenity_context().http, msg).await?;

        Ok(format!(
            "メッセージID: {} を編集しました。\n",
            data.message_id
        ))
    } else {
        Err("ギルドチャンネルではありません。".into())
    }
}

/// 掲示板のEmbedとボタンを作成する
async fn create_board_embed_and_buttons(
    ctx: &Context<'_>,
    chart_exists: bool,
) -> Result<(CreateEmbed, CreateActionRow), Error> {
    let now = chrono::Utc::now();
    let date = now.date_naive();
    let date_str = date.format("%m/%d").to_string();

    // 曜日の取得
    let weekday = now.weekday();
    let weekday_str = get_weekday_string(weekday);

    // ボタンの作成
    let found_button = CreateButton::new("found")
        .label("営業してる")
        .style(ButtonStyle::Primary);
    let not_found_button = CreateButton::new("not_found")
        .label("いない")
        .style(ButtonStyle::Secondary);
    let sold_out_button = CreateButton::new("sold_out")
        .label("売り切れた")
        .style(ButtonStyle::Danger);
    let action_row =
        CreateActionRow::Buttons(vec![found_button, not_found_button, sold_out_button]);

    // 投票結果を並行して取得
    let (found_count, not_found_count, sold_out_count) = tokio::try_join!(
        VoteService::count_votes_by_action(&ctx.data().database, "found".to_string()),
        VoteService::count_votes_by_action(&ctx.data().database, "not_found".to_string()),
        VoteService::count_votes_by_action(&ctx.data().database, "sold_out".to_string()),
    )?;

    // 最新の投票更新日時を取得
    let last_vote_updated_at = VoteService::get_latest_vote_updated_at(&ctx.data().database)
        .await?
        .unwrap_or(now);

    let mut embed = CreateEmbed::new()
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
            last_vote_updated_at.timestamp()
        ))
        .colour(Colour::from_rgb(0, 255, 0))
        .timestamp(now);

    // チャートが存在する場合はEmbedに画像を設定
    if chart_exists {
        embed = embed.image("attachment://vote_timeline.png");
    }

    Ok((embed, action_row))
}

/// 曜日を日本語文字列に変換する
fn get_weekday_string(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Mon => "月",
        chrono::Weekday::Tue => "火",
        chrono::Weekday::Wed => "水",
        chrono::Weekday::Thu => "木",
        chrono::Weekday::Fri => "金",
        chrono::Weekday::Sat => "土",
        chrono::Weekday::Sun => "日",
    }
}
