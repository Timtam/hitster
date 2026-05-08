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
use rocket_db_pools::sqlx::Row;
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

    pub async fn get_all_hit_stats(&self) -> HashMap<Uuid, HitStatsResponse> {
        let Some(pool) = self.pool.get() else {
            return HashMap::new();
        };
        let rows = match sqlx::query(
            "SELECT hit_id, correct_guesses, skips, tokens_earned, reveals FROM hit_stats",
        )
        .fetch_all(pool)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                rocket::error!("get_all_hit_stats: query failed: {}", e);
                return HashMap::new();
            }
        };
        rows.into_iter()
            .filter_map(|row| {
                let hit_id: Uuid = row.try_get("hit_id").ok()?;
                Some((
                    hit_id,
                    HitStatsResponse {
                        correct_guesses: row.try_get("correct_guesses").unwrap_or(0),
                        skips: row.try_get("skips").unwrap_or(0),
                        tokens_earned: row.try_get("tokens_earned").unwrap_or(0),
                        reveals: row.try_get("reveals").unwrap_or(0),
                    },
                ))
            })
            .collect()
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
            "SELECT games_played, games_won, hits_guessed_correctly, hits_guessed_wrong, \
             hits_stolen_successfully, hits_steal_attempts_failed, \
             tokens_earned, tokens_missed FROM user_stats WHERE user_id = ?",
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
            VirtualUserField::HitsGuessedWrong => entry.hits_guessed_wrong += 1,
            VirtualUserField::HitsStolenSuccessfully => entry.hits_stolen_successfully += 1,
            VirtualUserField::HitsStealAttemptsFailed => entry.hits_steal_attempts_failed += 1,
            VirtualUserField::TokensEarned => entry.tokens_earned += 1,
            VirtualUserField::TokensMissed => entry.tokens_missed += 1,
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

    pub fn record_user_hit_guessed_wrong(&self, user_id: Uuid, is_virtual: bool) {
        self.record_user(user_id, is_virtual, VirtualUserField::HitsGuessedWrong);
    }

    pub fn record_user_hit_stolen_successfully(&self, user_id: Uuid, is_virtual: bool) {
        self.record_user(user_id, is_virtual, VirtualUserField::HitsStolenSuccessfully);
    }

    pub fn record_user_hit_steal_attempt_failed(&self, user_id: Uuid, is_virtual: bool) {
        self.record_user(user_id, is_virtual, VirtualUserField::HitsStealAttemptsFailed);
    }

    pub fn record_user_token_earned(&self, user_id: Uuid, is_virtual: bool) {
        self.record_user(user_id, is_virtual, VirtualUserField::TokensEarned);
    }

    pub fn record_user_token_missed(&self, user_id: Uuid, is_virtual: bool) {
        self.record_user(user_id, is_virtual, VirtualUserField::TokensMissed);
    }
}

#[derive(Clone, Copy)]
enum VirtualUserField {
    GamesPlayed,
    GamesWon,
    HitsGuessedCorrectly,
    HitsGuessedWrong,
    HitsStolenSuccessfully,
    HitsStealAttemptsFailed,
    TokensEarned,
    TokensMissed,
}

impl VirtualUserField {
    fn column(self) -> &'static str {
        match self {
            VirtualUserField::GamesPlayed => "games_played",
            VirtualUserField::GamesWon => "games_won",
            VirtualUserField::HitsGuessedCorrectly => "hits_guessed_correctly",
            VirtualUserField::HitsGuessedWrong => "hits_guessed_wrong",
            VirtualUserField::HitsStolenSuccessfully => "hits_stolen_successfully",
            VirtualUserField::HitsStealAttemptsFailed => "hits_steal_attempts_failed",
            VirtualUserField::TokensEarned => "tokens_earned",
            VirtualUserField::TokensMissed => "tokens_missed",
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
