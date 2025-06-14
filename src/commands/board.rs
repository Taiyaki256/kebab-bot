use crate::{Context, Error, services::*};
use chrono::Datelike;
use poise::{
    CreateReply,
    serenity_prelude::{
        ButtonStyle, Colour, CreateActionRow, CreateButton, CreateEmbed, EditMessage,
    },
};

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
            .get_channel(poise::serenity_prelude::ChannelId::new(
                data.channel_id as u64,
            ))
            .await
        {
            if let poise::serenity_prelude::Channel::Guild(channel) = channel {
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

                    // 最新の投票更新日時を取得（なければ現在時刻を使用）
                    let last_vote_updated_at =
                        VoteService::get_latest_vote_updated_at(&ctx.data().database)
                            .await
                            .unwrap_or(None)
                            .unwrap_or(now);

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
                            last_vote_updated_at.timestamp()
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
