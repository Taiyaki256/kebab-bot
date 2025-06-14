use crate::entities::{vote, vote::Model as VoteModel};
use chrono::{NaiveDate, Timelike, Utc};
use chrono_tz::Asia::Tokyo;
use sea_orm::*;

pub struct VoteService;

impl VoteService {
    pub async fn create_vote(
        db: &DatabaseConnection,
        user_id: i64,
        action: String,
    ) -> Result<VoteModel, DbErr> {
        let now = Utc::now().into();

        let vote = vote::ActiveModel {
            user_id: Set(user_id),
            action: Set(action),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        vote.insert(db).await
    }

    pub async fn update_vote(
        db: &DatabaseConnection,
        user_id: i64,
        action: String,
    ) -> Result<VoteModel, DbErr> {
        let now = Utc::now().into();

        // 既存の投票を取得
        let existing_vote = vote::Entity::find_by_id(user_id).one(db).await?;

        // 既存の投票があれば更新、なければ新規作成
        if let Some(vote) = existing_vote {
            let mut vote: vote::ActiveModel = vote.into();
            vote.action = Set(action);
            vote.updated_at = Set(now);
            return vote.update(db).await;
        }

        Self::create_vote(db, user_id, action).await
    }

    pub async fn get_vote_by_action(
        db: &DatabaseConnection,
        action: String,
    ) -> Result<Vec<VoteModel>, DbErr> {
        vote::Entity::find()
            .filter(vote::Column::Action.eq(action))
            .all(db)
            .await
    }

    pub async fn count_votes_by_action(
        db: &DatabaseConnection,
        action: String,
    ) -> Result<u64, DbErr> {
        vote::Entity::find()
            .filter(vote::Column::Action.eq(action))
            .count(db)
            .await
    }

    /// 全ての投票データを取得
    pub async fn get_all_votes(db: &DatabaseConnection) -> Result<Vec<VoteModel>, DbErr> {
        vote::Entity::find()
            .order_by_asc(vote::Column::UpdatedAt)
            .all(db)
            .await
    }

    /// 特定の日付範囲での投票データを取得
    pub async fn get_votes_in_range(
        db: &DatabaseConnection,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<VoteModel>, DbErr> {
        vote::Entity::find()
            .filter(vote::Column::UpdatedAt.between(start_date, end_date))
            .order_by_asc(vote::Column::UpdatedAt)
            .all(db)
            .await
    }

    pub async fn delete_all_vote(db: &DatabaseConnection) -> Result<DeleteResult, DbErr> {
        vote::Entity::delete_many().exec(db).await
    }

    /// 最新の投票更新日時を取得
    pub async fn get_latest_vote_updated_at(
        db: &DatabaseConnection,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DbErr> {
        vote::Entity::find()
            .order_by_desc(vote::Column::UpdatedAt)
            .one(db)
            .await
            .map(|vote| vote.map(|v| v.updated_at.naive_utc().and_utc()))
    }

    /// 日本時間での現在の投票期間（午後期間）を取得
    /// 午後12時（正午）から午後11時59分59秒までを1つの投票期間とする
    pub fn get_current_jst_afternoon_period() -> NaiveDate {
        let now_jst = Utc::now().with_timezone(&Tokyo);

        // 午後12時（正午）より前の場合は前日の午後期間とみなす
        if now_jst.hour() < 12 {
            now_jst
                .date_naive()
                .pred_opt()
                .unwrap_or(now_jst.date_naive())
        } else {
            now_jst.date_naive()
        }
    }

    /// 最新の投票の日本時間での投票期間を取得
    pub async fn get_latest_vote_jst_afternoon_period(
        db: &DatabaseConnection,
    ) -> Result<Option<NaiveDate>, DbErr> {
        if let Some(dt) = Self::get_latest_vote_updated_at(db).await? {
            let jst_dt = dt.with_timezone(&Tokyo);

            // 午後12時（正午）より前の場合は前日の午後期間とみなす
            let period_date = if jst_dt.hour() < 12 {
                jst_dt
                    .date_naive()
                    .pred_opt()
                    .unwrap_or(jst_dt.date_naive())
            } else {
                jst_dt.date_naive()
            };

            Ok(Some(period_date))
        } else {
            Ok(None)
        }
    }

    /// 投票期間が変わったかどうかをチェックし、変わっていた場合は投票をリセット
    /// 午後12時（正午）を境に投票期間が切り替わる
    pub async fn check_and_reset_votes_if_new_day(db: &DatabaseConnection) -> Result<bool, DbErr> {
        let current_period = Self::get_current_jst_afternoon_period();
        let latest_vote_period = Self::get_latest_vote_jst_afternoon_period(db).await?;

        match latest_vote_period {
            Some(latest_period) if latest_period < current_period => {
                // 投票期間が変わっているので投票をリセット
                Self::delete_all_vote(db).await?;
                println!(
                    "🔄 投票期間が変わったため投票をリセットしました: {} → {}",
                    latest_period, current_period
                );
                Ok(true)
            }
            None => {
                // 投票データがない場合（初回起動など）
                println!("ℹ️ 投票データがありません（初回起動または既にリセット済み）");
                Ok(false)
            }
            Some(_) => {
                // 同じ投票期間なのでリセットしない
                Ok(false)
            }
        }
    }
}
