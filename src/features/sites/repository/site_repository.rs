use crate::core::dto::pagination::Items;
use crate::core::repository::paginate::paginate;
use crate::features::sites::model::prelude::Site;
use crate::features::sites::model::site;
use crate::features::sites::model::site::{Column, Model};
use crate::features::sites::validation::site_form::SiteForm;
use crate::features::users::model::{api_key, user};
use crate::utility::state::app_state;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, QueryTrait, Set,
};
use std::ops::Not;
use std::sync::Arc;

pub struct SiteRepository;

impl SiteRepository {
    pub async fn list(
        page: u64,
        per_page: u64,
        user: Arc<user::Model>,
        api_key: Arc<api_key::Model>,
    ) -> Result<Items<Model>, DbErr> {
        let state = app_state();

        let q = Site::find()
            .apply_if(user.is_admin.not().then_some(()), |q, _| {
                q.filter(Column::UserId.eq(user.id))
                    .filter(Column::ApiKeyId.eq(api_key.id))
            })
            .order_by_desc(Column::Id);

        paginate::<site::Entity>(q, &state._db, page, per_page).await
    }
    pub async fn list_by_user(
        user_id: i64,
        page: u64,
        per_page: u64,
    ) -> Result<Items<Model>, DbErr> {
        let state = app_state();
        let q = Site::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::Id);
        paginate::<site::Entity>(q, &state._db, page, per_page).await
    }

    pub async fn list_by_api_key(
        api_key_id: i64,
        page: u64,
        per_page: u64,
    ) -> Result<Items<Model>, DbErr> {
        let state = app_state();
        let q = Site::find()
            .filter(Column::ApiKeyId.eq(api_key_id))
            .order_by_desc(Column::Id);
        paginate::<site::Entity>(q, &state._db, page, per_page).await
    }

    pub async fn list_all_by_api_key(api_key_id: i64) -> Result<Vec<Model>, DbErr> {
        let state = app_state();

        Site::find()
            .filter(Column::ApiKeyId.eq(api_key_id))
            .order_by_desc(Column::Id)
            .all(&state._db)
            .await
    }

    pub async fn list_all_by_user_id(id: i64) -> Result<Vec<Model>, DbErr> {
        let state = app_state();

        Site::find()
            .filter(Column::UserId.eq(id))
            .order_by_desc(Column::Id)
            .all(&state._db)
            .await
    }

    pub async fn all() -> Result<Vec<Model>, DbErr> {
        let state = app_state();
        Site::find()
            .filter(Column::Status.eq(true))
            .order_by_desc(Column::Id)
            .all(&state._db)
            .await
    }

    pub async fn create(data: SiteForm) -> Result<Option<Model>, DbErr> {
        let state = app_state();

        let user_id = data.user_id.expect("user_id must be set before create");
        let api_key_id = data
            .api_key_id
            .expect("api_key_id must be set before create");

        let am = site::ActiveModel {
            name: Set(data.name),
            url: Set(data.url),
            url_list: Set(data.url_list),
            path_link: Set(data.path_link),
            path_title: Set(data.path_title),
            path_content: Set(data.path_content),
            path_image: Set(data.path_image),
            path_video: Set(data.path_video),
            path_remove: Set(data.path_remove),
            screenshot: Set(data.screenshot),
            status: Set(data.status),
            user_id: Set(user_id),
            api_key_id: Set(api_key_id),
            ..Default::default()
        };

        match am.insert(&state._db).await {
            Ok(model) => Ok(Some(model)),
            Err(err) => Err(err),
        }
    }

    pub async fn update(site_id: i64, data: SiteForm) -> Result<Option<Model>, DbErr> {
        let state = app_state();

        let Some(_existing) = site::Entity::find_by_id(site_id).one(&state._db).await? else {
            return Ok(None);
        };

        let am = site::ActiveModel {
            id: Set(site_id),
            name: Set(data.name),
            url: Set(data.url),
            url_list: Set(data.url_list),
            path_link: Set(data.path_link),
            path_title: Set(data.path_title),
            path_content: Set(data.path_content),
            path_image: Set(data.path_image),
            path_video: Set(data.path_video),
            path_remove: Set(data.path_remove),
            screenshot: Set(data.screenshot),
            status: Set(data.status),
            ..Default::default()
        };

        let updated = am.update(&state._db).await?;
        Ok(Some(updated))
    }

    pub async fn disable(site_id: i64) -> Result<Option<Model>, DbErr> {
        let state = app_state();

        let Some(_existing) = site::Entity::find_by_id(site_id).one(&state._db).await? else {
            return Ok(None);
        };

        let am = site::ActiveModel {
            id: Set(site_id),
            status: Set(Option::from(false)),
            ..Default::default()
        };

        let updated = am.update(&state._db).await?;
        Ok(Some(updated))
    }

    pub async fn delete(site_id: i64) -> Result<bool, String> {
        let state = app_state();

        let Some(_site) = site::Entity::find_by_id(site_id)
            .one(&state._db)
            .await
            .map_err(|e| e.to_string())?
        else {
            return Err("Site not found".to_string());
        };

        match site::Entity::delete_by_id(site_id).exec(&state._db).await {
            Ok(_) => Ok(true),
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn find_by_id(site_id: i64) -> Result<Option<Model>, DbErr> {
        let state = app_state();

        let site = site::Entity::find_by_id(site_id).one(&state._db).await?;
        Ok(site)
    }
}
