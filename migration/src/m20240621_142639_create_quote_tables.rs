use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create quote_price table
        manager.create_table(
            Table::create()
                .table(QuotePrice::Table)
                .if_not_exists()
                .col(ColumnDef::new(QuotePrice::Symbol).string().not_null())
                .col(ColumnDef::new(QuotePrice::Sequence).big_integer().not_null())
                .col(ColumnDef::new(QuotePrice::LastDone).decimal().not_null())
                .col(ColumnDef::new(QuotePrice::Open).decimal().not_null())
                .col(ColumnDef::new(QuotePrice::High).decimal().not_null())
                .col(ColumnDef::new(QuotePrice::Low).decimal().not_null())
                .col(ColumnDef::new(QuotePrice::Timestamp).big_integer().not_null())
                .col(ColumnDef::new(QuotePrice::Volume).big_integer().not_null())
                .col(ColumnDef::new(QuotePrice::Turnover).decimal().not_null())
                .col(ColumnDef::new(QuotePrice::TradeStatus).integer().not_null())
                .col(ColumnDef::new(QuotePrice::TradeSession).integer().not_null())
                .col(ColumnDef::new(QuotePrice::CurrentVolume).big_integer().not_null())
                .col(ColumnDef::new(QuotePrice::CurrentTurnover).decimal().not_null())
                .col(ColumnDef::new(QuotePrice::Tag).integer().not_null())
                .to_owned(),
        ).await?;

        // Create quote_trade table
        manager.create_table(
            Table::create()
                .table(QuoteTrade::Table)
                .if_not_exists()
                .col(ColumnDef::new(QuoteTrade::Symbol).string().not_null())
                .col(ColumnDef::new(QuoteTrade::Sequence).big_integer().not_null())
                .col(ColumnDef::new(QuoteTrade::Price).decimal().not_null())
                .col(ColumnDef::new(QuoteTrade::Volume).integer().not_null())
                .col(ColumnDef::new(QuoteTrade::Timestamp).big_integer().not_null())
                .col(ColumnDef::new(QuoteTrade::TradeType).char().not_null())
                .col(ColumnDef::new(QuoteTrade::Direction).integer().not_null())
                .col(ColumnDef::new(QuoteTrade::TradeSession).integer().not_null())
                .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(QuotePrice::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(QuoteTrade::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum QuotePrice {
    Table,
    Id,
    Symbol,
    Sequence,
    LastDone,
    Open,
    High,
    Low,
    Timestamp,
    Volume,
    Turnover,
    TradeStatus,
    TradeSession,
    CurrentVolume,
    CurrentTurnover,
    Tag,
}

#[derive(DeriveIden)]
pub enum QuoteTrade {
    Table,
    Id,
    Symbol,
    Sequence,
    Price,
    Volume,
    Timestamp,
    TradeType,
    Direction,
    TradeSession,
}