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
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};
use uuid::Uuid;

use super::ServiceStore;

#[derive(Default)]
pub struct StatsService {
    pool: OnceLock<SqlitePool>,
    virtual_user_stats: Mutex<HashMap<Uuid, UserStatsResponse>>,
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
        if let Some(stats) = self.virtual_user_stats.lock().unwrap().get(&user_id) {
            return stats.clone();
        }

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
            rocket::warn!(
                "spawn_bump_user: pool not set yet, dropping write user_id={} col={}",
                user_id,
                column
            );
            return;
        };
        rocket::debug!(
            "spawn_bump_user: spawning user_id={} col={}",
            user_id,
            column
        );
        rocket::tokio::spawn(async move {
            let sql = format!(
                "INSERT INTO user_stats (user_id, {col}) VALUES (?, 1) \
                 ON CONFLICT(user_id) DO UPDATE SET {col} = {col} + 1",
                col = column,
            );
            match sqlx::query(&sql).bind(user_id).execute(&pool).await {
                Ok(r) => rocket::debug!(
                    "spawn_bump_user: OK user_id={} col={} rows={}",
                    user_id,
                    column,
                    r.rows_affected()
                ),
                Err(e) => rocket::error!(
                    "spawn_bump_user: ERROR user_id={} col={} err={}",
                    user_id,
                    column,
                    e
                ),
            }
        });
    }

    fn bump_virtual_user(&self, user_id: Uuid, field: VirtualUserField) {
        let mut map = self.virtual_user_stats.lock().unwrap();
        let entry = map.entry(user_id).or_default();
        match field {
            VirtualUserField::GamesPlayed => entry.games_played += 1,
            VirtualUserField::GamesWon => entry.games_won += 1,
            VirtualUserField::HitsGuessedCorrectly => entry.hits_guessed_correctly += 1,
            VirtualUserField::TokensEarned => entry.tokens_earned += 1,
        }
    }

    pub fn drop_virtual_user_stats(&self, user_id: Uuid) {
        self.virtual_user_stats.lock().unwrap().remove(&user_id);
    }

    fn record_user(&self, user_id: Uuid, is_virtual: bool, field: VirtualUserField) {
        if is_virtual {
            self.bump_virtual_user(user_id, field);
        } else {
            self.spawn_bump_user(user_id, field.column());
        }
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

    pub fn record_user_game_played(&self, user_id: Uuid, is_virtual: bool) {
        self.record_user(user_id, is_virtual, VirtualUserField::GamesPlayed);
    }

    pub fn record_user_game_won(&self, user_id: Uuid, is_virtual: bool) {
        self.record_user(user_id, is_virtual, VirtualUserField::GamesWon);
    }

    pub fn record_user_hit_guessed_correctly(&self, user_id: Uuid, is_virtual: bool) {
        self.record_user(user_id, is_virtual, VirtualUserField::HitsGuessedCorrectly);
    }

    pub fn record_user_token_earned(&self, user_id: Uuid, is_virtual: bool) {
        self.record_user(user_id, is_virtual, VirtualUserField::TokensEarned);
    }
}

#[derive(Clone, Copy)]
enum VirtualUserField {
    GamesPlayed,
    GamesWon,
    HitsGuessedCorrectly,
    TokensEarned,
}

impl VirtualUserField {
    fn column(self) -> &'static str {
        match self {
            VirtualUserField::GamesPlayed => "games_played",
            VirtualUserField::GamesWon => "games_won",
            VirtualUserField::HitsGuessedCorrectly => "hits_guessed_correctly",
            VirtualUserField::TokensEarned => "tokens_earned",
        }
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
