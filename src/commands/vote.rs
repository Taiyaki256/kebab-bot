use crate::entities::vote::Model as VoteModel;
use crate::{Context, Error, services::*};
use poise::{
    CreateReply,
    serenity_prelude::{Colour, CreateEmbed},
};

/// 投票をリセットするコマンド
#[poise::command(slash_command)]
pub async fn reset_votes(ctx: Context<'_>) -> Result<(), Error> {
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
pub async fn vote_results(ctx: Context<'_>) -> Result<(), Error> {
    // 日付チェックを行い、必要に応じて投票をリセット
    if let Err(e) = VoteService::check_and_reset_votes_if_new_day(&ctx.data().database).await {
        eprintln!("日付チェック中にエラーが発生しました: {}", e);
    }

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
pub async fn vote_chart(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    // 日付チェックを行い、必要に応じて投票をリセット
    if let Err(e) = VoteService::check_and_reset_votes_if_new_day(&ctx.data().database).await {
        eprintln!("日付チェック中にエラーが発生しました: {}", e);
    }

    // 投票データを取得
    let votes = VoteService::get_all_votes(&ctx.data().database).await?;

    if votes.is_empty() {
        ctx.say("📊 まだ投票データがありません。").await?;
        return Ok(());
    }

    // 時系列グラフを生成
    let timeline_path = "vote_timeline.png";
    match ChartService::generate_vote_timeline_chart(votes, timeline_path).await {
        Ok(_) => {
            // ファイルを送信
            let file = poise::serenity_prelude::CreateAttachment::path(timeline_path).await?;
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

    Ok(())
}
