use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    // Unique id for user
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    // Discord id for user
    pub discord_id: i64,

    // User's Minecraft username
    pub minecraft_username: Option<String>,

}

impl ActiveModelBehavior for ActiveModel {}

