use crate::entities::board_data::{self, Model as BoardDataModel};
use crate::entities::prelude::*;
use chrono::Utc;
use sea_orm::*;

pub struct BoardService;

impl BoardService {
    /// 新しいボードデータを作成
    pub async fn create_board_data(
        db: &DatabaseConnection,
        server_id: i64,
        channel_id: i64,
        message_id: i64,
    ) -> Result<BoardDataModel, DbErr> {
        let now = Utc::now().into();

        let board_data = board_data::ActiveModel {
            server_id: Set(server_id),
            channel_id: Set(channel_id),
            message_id: Set(message_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        board_data.insert(db).await
    }

    /// IDでボードデータを取得
    pub async fn get_board_data_by_id(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<Option<BoardDataModel>, DbErr> {
        BoardData::find_by_id(id).one(db).await
    }

    /// サーバーIDでボードデータを取得
    pub async fn get_board_data_by_server_id(
        db: &DatabaseConnection,
        server_id: i64,
    ) -> Result<Vec<BoardDataModel>, DbErr> {
        BoardData::find()
            .filter(board_data::Column::ServerId.eq(server_id))
            .all(db)
            .await
    }

    /// チャンネルIDでボードデータを取得
    pub async fn get_board_data_by_channel_id(
        db: &DatabaseConnection,
        channel_id: i64,
    ) -> Result<Vec<BoardDataModel>, DbErr> {
        BoardData::find()
            .filter(board_data::Column::ChannelId.eq(channel_id))
            .all(db)
            .await
    }

    /// メッセージIDでボードデータを取得
    pub async fn get_board_data_by_message_id(
        db: &DatabaseConnection,
        message_id: i64,
    ) -> Result<Option<BoardDataModel>, DbErr> {
        BoardData::find()
            .filter(board_data::Column::MessageId.eq(message_id))
            .one(db)
            .await
    }

    /// ボードデータを更新
    pub async fn update_board_data(
        db: &DatabaseConnection,
        id: i32,
        server_id: Option<i64>,
        channel_id: Option<i64>,
        message_id: Option<i64>,
    ) -> Result<BoardDataModel, DbErr> {
        let board_data = BoardData::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Board data not found".to_string()))?;

        let mut board_data: board_data::ActiveModel = board_data.into();

        if let Some(server_id) = server_id {
            board_data.server_id = Set(server_id);
        }
        if let Some(channel_id) = channel_id {
            board_data.channel_id = Set(channel_id);
        }
        if let Some(message_id) = message_id {
            board_data.message_id = Set(message_id);
        }

        board_data.updated_at = Set(Utc::now().into());

        board_data.update(db).await
    }

    /// ボードデータを削除
    pub async fn delete_board_data(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<DeleteResult, DbErr> {
        let board_data = BoardData::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Board data not found".to_string()))?;

        board_data.delete(db).await
    }

    /// 全てのボードデータを取得
    pub async fn get_all_board_data(db: &DatabaseConnection) -> Result<Vec<BoardDataModel>, DbErr> {
        BoardData::find().all(db).await
    }

    /// ボードデータの総数を取得
    pub async fn count_board_data(db: &DatabaseConnection) -> Result<u64, DbErr> {
        BoardData::find().count(db).await
    }

    /// 特定の日付以降に作成されたボードデータを取得
    pub async fn get_board_data_since(
        db: &DatabaseConnection,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<BoardDataModel>, DbErr> {
        BoardData::find()
            .filter(board_data::Column::CreatedAt.gte(since))
            .order_by_desc(board_data::Column::CreatedAt)
            .all(db)
            .await
    }

    /// 特定のサーバーとチャンネルの組み合わせでボードデータを検索
    pub async fn get_board_data_by_server_and_channel(
        db: &DatabaseConnection,
        server_id: i64,
        channel_id: i64,
    ) -> Result<Vec<BoardDataModel>, DbErr> {
        BoardData::find()
            .filter(board_data::Column::ServerId.eq(server_id))
            .filter(board_data::Column::ChannelId.eq(channel_id))
            .order_by_desc(board_data::Column::CreatedAt)
            .all(db)
            .await
    }
}
