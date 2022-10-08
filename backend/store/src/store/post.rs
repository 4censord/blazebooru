use anyhow::Context;
use blazebooru_models::view as vm;

use crate::{models as dbm, PgStore, StoreError};

impl PgStore {
    pub async fn get_post(&self, id: i32) -> Result<Option<dbm::Post>, StoreError> {
        let post = sqlx::query_as_unchecked!(dbm::Post, r#"SELECT * FROM post WHERE id = $1;"#, id)
            .fetch_optional(&self.pool)
            .await
            .context("Error getting post from database")?;

        Ok(post)
    }

    pub async fn create_post(&self, post: &dbm::NewPost) -> Result<dbm::Post, StoreError> {
        let post = sqlx::query_as_unchecked!(dbm::Post, r#"SELECT * FROM create_post($1);"#, post)
            .fetch_one(&self.pool)
            .await
            .context("Error creating post in database")?;

        Ok(post)
    }

    pub async fn get_view_post(&self, id: i32) -> Result<Option<dbm::ViewPost>, StoreError> {
        let post = sqlx::query_as_unchecked!(
            dbm::ViewPost,
            r#"SELECT * FROM view_post WHERE id = $1;"#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Error getting view post from database")?;

        Ok(post)
    }

    pub async fn get_view_posts(
        &self,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<dbm::ViewPost>, StoreError> {
        let posts = sqlx::query_as_unchecked!(
            dbm::ViewPost,
            r#"SELECT * FROM view_post ORDER BY id DESC LIMIT $1 OFFSET $2;"#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Error getting view posts from database")?;

        Ok(posts)
    }

    pub async fn get_posts_pagination_stats(&self) -> Result<vm::PaginationStats, StoreError> {
        let stats = sqlx::query_as_unchecked!(
            vm::PaginationStats,
            r#"SELECT MAX(id) AS max_id, COUNT(*)::integer AS count FROM post;"#
        )
        .fetch_one(&self.pool)
        .await
        .context("Error getting post pagination stats from database")?;

        Ok(stats)
    }
}