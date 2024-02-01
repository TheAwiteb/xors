// A RESTful tic tac toy API for XORS project
// Copyright (C) 2024  Awiteb <awitb@hotmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Game::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Game::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Game::Uuid).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Game::Round).small_integer().not_null())
                    .col(ColumnDef::new(Game::AutoPlayAfter).date_time())
                    .col(ColumnDef::new(Game::RoundsResult).string().not_null())
                    .col(ColumnDef::new(Game::XPlayer).uuid().not_null())
                    .col(ColumnDef::new(Game::OPlayer).uuid().not_null())
                    .col(ColumnDef::new(Game::Board).string().not_null())
                    .col(ColumnDef::new(Game::Winner).uuid())
                    .col(ColumnDef::new(Game::Reason).string())
                    .col(ColumnDef::new(Game::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Game::EndedAt).date_time())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Game::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Game {
    Table,
    Id,
    Uuid,
    Round,
    AutoPlayAfter,
    RoundsResult,
    XPlayer,
    OPlayer,
    XStartChat,
    OStartChat,
    Board,
    Winner,
    Reason,
    CreatedAt,
    EndedAt,
}
