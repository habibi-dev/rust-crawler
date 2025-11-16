use crate::core::dto::pagination::Items;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, Select};

pub async fn paginate<E>(
    query: Select<E>,
    db: &DatabaseConnection,
    page: u64,
    per_page: u64,
) -> Result<Items<E::Model>, DbErr>
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
{
    let per: usize = per_page.max(1) as usize;
    let page0: usize = page.saturating_sub(1) as usize;

    let paginator = Select::paginate(query, db, per as u64);
    let total = paginator.num_items().await?;
    let data = paginator.fetch_page(page0 as u64).await?;

    Ok(Items::new(data, page, per_page, total))
}
