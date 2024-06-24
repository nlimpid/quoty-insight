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
                .col(
                    ColumnDef::new(QuotePrice::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(QuotePrice::Symbol).string().not_null())
                .col(ColumnDef::new(QuotePrice::Sequence).big_integer().default(0))
                .col(ColumnDef::new(QuotePrice::LastDone).decimal())
                .col(ColumnDef::new(QuotePrice::Open).decimal())
                .col(ColumnDef::new(QuotePrice::High).decimal())
                .col(ColumnDef::new(QuotePrice::Low).decimal())
                .col(ColumnDef::new(QuotePrice::Timestamp).big_integer().not_null())
                .col(ColumnDef::new(QuotePrice::Volume).big_integer())
                .col(ColumnDef::new(QuotePrice::Turnover).decimal())
                .col(ColumnDef::new(QuotePrice::TradeStatus).integer())
                .col(ColumnDef::new(QuotePrice::TradeSession).integer())
                .to_owned(),
        ).await?;

        // Create quote_trade table
        manager.create_table(
            Table::create()
                .table(QuoteTrade::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(QuotePrice::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(QuoteTrade::Symbol).string().not_null())
                .col(ColumnDef::new(QuoteTrade::Sequence).big_integer().not_null().default(0))
                .col(ColumnDef::new(QuoteTrade::Price).decimal())
                .col(ColumnDef::new(QuoteTrade::Volume).integer())
                .col(ColumnDef::new(QuoteTrade::Timestamp).big_integer().not_null())
                .col(ColumnDef::new(QuoteTrade::TradeType).char())
                .col(ColumnDef::new(QuoteTrade::Direction).integer())
                .col(ColumnDef::new(QuoteTrade::TradeSession).integer())
                .to_owned(),
        ).await?;


        // Create quote_trade table
        manager.create_table(
            Table::create()
                .table(QuoteSub::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(QuoteSub::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(QuoteSub::Symbol).string().not_null())
                .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(QuotePrice::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(QuoteTrade::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(QuoteSub::Table).to_owned()).await?;
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


#[derive(DeriveIden)]
pub enum QuoteSub {
    Table,
    Id,
    Symbol,
}