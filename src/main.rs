mod quote_server;
use migration::{Migrator, MigratorTrait};

fn main() {

    let connection = sea_orm::Database::connect(&database_url).await?;
    Migrator::up(&connection, None).await?;
}
