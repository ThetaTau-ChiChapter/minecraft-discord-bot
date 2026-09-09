use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "
            CREATE TABLE users (
                id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
                discord_id BIGINT NOT NULL,
                minecraft_username TEXT
            );

            COMMENT ON TABLE users IS 'Table for storing API keys';
            COMMENT ON COLUMN users.id IS 'Unique id for user';
            COMMENT ON COLUMN users.discord_id IS 'Discord id for user';
            COMMENT ON COLUMN users.minecraft_username IS 'Minecraft username associated with account';

            CREATE TABLE pending_user_codes (
                id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
                code TEXT NOT NULL UNIQUE,
                minecraft_username TEXT NOT NULL UNIQUE
            );

            COMMENT ON TABLE pending_user_codes IS 'Pending codes for users registering their minecraft account';
            COMMENT ON COLUMN pending_user_codes.id IS 'Unique id for referencing code';
            COMMENT ON COLUMN pending_user_codes.code IS 'Unique code that user must enter to register';
            COMMENT ON COLUMN pending_user_codes.minecraft_username IS 'Users minecraft username associated with code';
            ",
        )
        .await?;

        Ok(())
    }
}
