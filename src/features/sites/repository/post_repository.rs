use crate::core::dto::pagination::Items;
use crate::core::repository::paginate::paginate;
use crate::core::state::AppState;
use crate::features::sites::model::posts::{Column, Model, PostStatus};
use crate::features::sites::model::prelude::Posts;
use crate::features::sites::model::{posts, site};
use crate::features::sites::validation::post_form::{PostForm, PostFormCreate};
use crate::utility::state::app_state;
use sea_orm::ColumnTrait;
use sea_orm::{
    ActiveModelTrait, DbErr, DeleteResult, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};

pub struct PostRepository;

impl PostRepository {
    pub async fn list(page: u64, per_page: u64, post_id: u64) -> Result<Items<Model>, DbErr> {
        let state = app_state();
        let q = Posts::find()
            .filter(Column::Id.gt(post_id))
            .order_by_desc(Column::Id);
        paginate::<posts::Entity>(q, &state._db, page, per_page).await
    }

    pub async fn list_by_site(
        site_id: i64,
        page: u64,
        per_page: u64,
        post_id: u64,
    ) -> Result<Items<Model>, DbErr> {
        let state = app_state();
        let q = Posts::find()
            .filter(Column::SiteId.eq(site_id))
            .filter(Column::Id.gt(post_id))
            .order_by_desc(Column::Id);
        paginate::<posts::Entity>(q, &state._db, page, per_page).await
    }

    pub async fn list_by_user(
        user_id: i64,
        page: u64,
        per_page: u64,
        post_id: u64,
    ) -> Result<Items<Model>, DbErr> {
        let state = app_state();
        let q = Posts::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Id.gt(post_id))
            .order_by_desc(Column::Id);
        paginate::<posts::Entity>(q, &state._db, page, per_page).await
    }

    pub async fn list_by_api_key(
        api_key_id: i64,
        page: u64,
        per_page: u64,
        post_id: u64,
    ) -> Result<Items<Model>, DbErr> {
        let state = app_state();
        let q = Posts::find()
            .filter(Column::ApiKeyId.eq(api_key_id))
            .filter(Column::Id.gt(post_id))
            .order_by_desc(Column::Id);
        paginate::<posts::Entity>(q, &state._db, page, per_page).await
    }

    pub async fn pending_list() -> Result<Vec<(Model, site::Model)>, DbErr> {
        let state = app_state();
        let rows = Posts::find()
            .filter(Column::Status.is_in([PostStatus::PENDING, PostStatus::FAILED]))
            .order_by_desc(Column::Id)
            .find_also_related(site::Entity)
            .all(&state._db)
            .await?;

        // If site is None → drop that row (defensive)
        let rows = rows
            .into_iter()
            .filter_map(|(post, site)| site.map(|s| (post, s)))
            .collect();

        Ok(rows)
    }

    pub async fn find_by_id(post_id: i64) -> Result<Option<Model>, DbErr> {
        let state = app_state();
        posts::Entity::find_by_id(post_id).one(&state._db).await
    }

    pub async fn find_by_url(url: &str) -> Result<Option<Model>, DbErr> {
        let state = app_state();
        posts::Entity::find()
            .filter(Column::Url.eq(url.to_string()))
            .one(&state._db)
            .await
    }

    pub async fn create(data: PostFormCreate) -> Result<Option<Model>, DbErr> {
        let state = app_state();
        let am = posts::ActiveModel {
            url: Set(data.url),
            site_id: Set(data.site_id),
            user_id: Set(data.user_id),
            api_key_id: Set(data.api_key_id),
            ..Default::default()
        };

        match am.insert(&state._db).await {
            Ok(model) => Ok(Some(model)),
            Err(e) => Err(e),
        }
    }

    pub async fn update(post_id: i64, data: PostForm) -> Result<Option<Model>, DbErr> {
        let state = app_state();

        let Some(existing) = Self::find_existing_post(state, post_id).await? else {
            return Ok(None);
        };

        let retry = Self::next_retry(existing.retry);
        let am = Self::build_content_active_model(post_id, retry, data);

        let updated = am.update(&state._db).await?;
        Ok(Some(updated))
    }

    pub async fn update_failed(post_id: i64) -> Result<Option<Model>, DbErr> {
        let state = app_state();

        let Some(existing) = Self::find_existing_post(state, post_id).await? else {
            return Ok(None);
        };

        let retry = Self::next_retry(existing.retry);
        let status = Self::resolve_failure_status(retry, state.config.max_retry_post as i8);
        let am = Self::build_failure_active_model(post_id, retry, status);

        let updated = am.update(&state._db).await?;
        Ok(Some(updated))
    }

    pub async fn delete(post_id: i64) -> Result<bool, String> {
        let state = app_state();

        let Some(_existing) = posts::Entity::find_by_id(post_id)
            .one(&state._db)
            .await
            .map_err(|e| e.to_string())?
        else {
            return Err("Post not found".to_string());
        };

        posts::Entity::delete_by_id(post_id)
            .exec(&state._db)
            .await
            .map(|_| true)
            .map_err(|e| e.to_string())
    }

    pub async fn cleanup_old_posts(keep_latest: u64) -> Result<u64, DbErr> {
        if keep_latest == 0 {
            return Ok(0);
        }

        let state = app_state();
        let offset = keep_latest.saturating_sub(1);
        let boundary_post = Posts::find()
            .order_by_desc(Column::Id)
            .offset(offset)
            .limit(1)
            .one(&state._db)
            .await?;

        let Some(boundary) = boundary_post else {
            return Ok(0);
        };

        let result: DeleteResult = Posts::delete_many()
            .filter(Column::Id.lt(boundary.id))
            .exec(&state._db)
            .await?;

        Ok(result.rows_affected)
    }

    async fn find_existing_post(state: &AppState, post_id: i64) -> Result<Option<Model>, DbErr> {
        posts::Entity::find_by_id(post_id).one(&state._db).await
    }

    fn next_retry(current_retry: i8) -> i8 {
        current_retry + 1
    }

    fn build_content_active_model(post_id: i64, retry: i8, data: PostForm) -> posts::ActiveModel {
        posts::ActiveModel {
            id: Set(post_id),
            title: Set(data.title),
            body: Set(data.body),
            image: Set(data.image),
            video: Set(data.video),
            status: Set(data.status),
            retry: Set(retry),
            ..Default::default()
        }
    }

    fn build_failure_active_model(
        post_id: i64,
        retry: i8,
        status: PostStatus,
    ) -> posts::ActiveModel {
        posts::ActiveModel {
            id: Set(post_id),
            status: Set(status),
            retry: Set(retry),
            ..Default::default()
        }
    }

    fn resolve_failure_status(retry: i8, max_retry: i8) -> PostStatus {
        if retry >= max_retry {
            PostStatus::CANCELLED
        } else {
            PostStatus::FAILED
        }
    }
}
