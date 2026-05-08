use crate::{
    HitsterConfig,
    responses::{HitStatsResponse, UserStatsResponse},
};
use rocket::{
    Orbit, Rocket,
    fairing::{Fairing, Info, Kind},
};
use rocket_db_pools::{
    Database,
    sqlx::{self, SqlitePool},
};
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

use super::ServiceStore;

#[derive(Default)]
pub struct StatsService {
    pool: OnceLock<SqlitePool>,
}

impl StatsService {
    pub fn set_pool(&self, pool: SqlitePool) {
        let _ = self.pool.set(pool);
    }

    pub async fn get_hit_stats(&self, hit_id: Uuid) -> HitStatsResponse {
        let Some(pool) = self.pool.get() else {
            return HitStatsResponse::default();
        };
        sqlx::query_as::<_, HitStatsResponse>(
            "SELECT correct_guesses, skips, tokens_earned, reveals \
             FROM hit_stats WHERE hit_id = ?",
        )
        .bind(hit_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
    }

    pub async fn get_user_stats(&self, user_id: Uuid) -> UserStatsResponse {
        let Some(pool) = self.pool.get() else {
            return UserStatsResponse::default();
        };
        sqlx::query_as::<_, UserStatsResponse>(
            "SELECT games_played, games_won, hits_guessed_correctly, tokens_earned \
             FROM user_stats WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
    }

    fn spawn_bump_hit(&self, hit_id: Uuid, column: &'static str) {
        let Some(pool) = self.pool.get().cloned() else {
            return;
        };
        rocket::tokio::spawn(async move {
            let sql = format!(
                "INSERT INTO hit_stats (hit_id, {col}) VALUES (?, 1) \
                 ON CONFLICT(hit_id) DO UPDATE SET {col} = {col} + 1",
                col = column,
            );
            let _ = sqlx::query(&sql).bind(hit_id).execute(&pool).await;
        });
    }

    fn spawn_bump_user(&self, user_id: Uuid, column: &'static str) {
        let Some(pool) = self.pool.get().cloned() else {
            return;
        };
        rocket::tokio::spawn(async move {
            let sql = format!(
                "INSERT INTO user_stats (user_id, {col}) VALUES (?, 1) \
                 ON CONFLICT(user_id) DO UPDATE SET {col} = {col} + 1",
                col = column,
            );
            let _ = sqlx::query(&sql).bind(user_id).execute(&pool).await;
        });
    }

    pub fn record_hit_correct_guess(&self, hit_id: Uuid) {
        self.spawn_bump_hit(hit_id, "correct_guesses");
    }

    pub fn record_hit_skip(&self, hit_id: Uuid) {
        self.spawn_bump_hit(hit_id, "skips");
    }

    pub fn record_hit_token_earned(&self, hit_id: Uuid) {
        self.spawn_bump_hit(hit_id, "tokens_earned");
    }

    pub fn record_hit_reveal(&self, hit_id: Uuid) {
        self.spawn_bump_hit(hit_id, "reveals");
    }

    pub fn record_user_game_played(&self, user_id: Uuid) {
        self.spawn_bump_user(user_id, "games_played");
    }

    pub fn record_user_game_won(&self, user_id: Uuid) {
        self.spawn_bump_user(user_id, "games_won");
    }

    pub fn record_user_hit_guessed_correctly(&self, user_id: Uuid) {
        self.spawn_bump_user(user_id, "hits_guessed_correctly");
    }

    pub fn record_user_token_earned(&self, user_id: Uuid) {
        self.spawn_bump_user(user_id, "tokens_earned");
    }
}

#[derive(Default)]
pub struct StatsServiceFairing;

#[rocket::async_trait]
impl Fairing for StatsServiceFairing {
    fn info(&self) -> Info {
        Info {
            name: "Inject DB pool into StatsService",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let pool = HitsterConfig::fetch(rocket).unwrap().0.clone();
        let stats_service: Arc<StatsService> =
            rocket.state::<ServiceStore>().unwrap().stats_service();
        stats_service.set_pool(pool);
    }
}
