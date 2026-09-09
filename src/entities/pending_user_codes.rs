use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "pending_user_codes")]
pub struct Model {
    // Unique id for user
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    // Discord id for user
    pub code: String,

    // User's Minecraft username
    pub minecraft_username: String,

}

impl ActiveModelBehavior for ActiveModel {}

