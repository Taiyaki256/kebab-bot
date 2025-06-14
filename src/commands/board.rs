use crate::{Context, Error, services::*};
use poise::CreateReply;

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
        BoardUIService::handle_empty_board_data(&ctx).await?;
        return Ok(());
    }

    let response = BoardUIService::update_all_board_messages(&ctx, board_data).await?;

    let rep = ctx
        .reply_builder(CreateReply::default())
        .content(response)
        .ephemeral(true);
    ctx.send(rep).await?;
    Ok(())
}
