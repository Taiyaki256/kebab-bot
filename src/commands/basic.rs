use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// ping コマンド
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// ユーザー情報を表示するコマンド
#[poise::command(slash_command)]
pub async fn userinfo(
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
pub async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
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
pub async fn help(
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
