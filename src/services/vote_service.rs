use crate::entities::prelude::*;
use crate::entities::{vote, vote::Model as VoteModel};
use chrono::Utc;
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
}
