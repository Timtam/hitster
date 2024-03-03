use crate::{
    responses::{GameResponse, GamesResponse, MessageResponse},
    services::{GameService, UserService},
    users::User,
};
use rocket::{
    response::status::{Created, NotFound},
    serde::json::Json,
    State,
};
use rocket_okapi::openapi;

/// Create a new game
///
/// Create a new game. The currently logged in user will be the creator of the game. The creator will be the only one who can change game properties later.
/// The API will return 401 if the user_id is invalid.

#[openapi(tag = "Games")]
#[post("/games")]
pub fn create_game(
    user: User,
    games: &State<GameService>,
    users: &State<UserService>,
) -> Created<Json<GameResponse>> {
    let game = games.add(user.id);

    Created::new(format!("/games/{}", game.id)).body(Json(GameResponse {
        id: game.id,
        creator: users.get_by_id(game.creator).unwrap(),
        players: game
            .players
            .into_iter()
            .map(|id| users.get_by_id(id).unwrap())
            .collect::<_>(),
    }))
}

/// Retrieve all currently known games
///
/// Get a flat overview over all current games.
/// The info returned by this call is currently identical to GET /games/game_id or POST /games/user_id calls, but that might change later.

#[openapi(tag = "Games")]
#[get("/games")]
pub fn get_all_games(
    games: &State<GameService>,
    users: &State<UserService>,
) -> Json<GamesResponse> {
    Json(GamesResponse {
        games: games
            .get_all()
            .into_iter()
            .map(|game| GameResponse {
                id: game.id,
                creator: users.get_by_id(game.creator).unwrap(),
                players: game
                    .players
                    .into_iter()
                    .map(|id| users.get_by_id(id).unwrap())
                    .collect::<_>(),
            })
            .collect::<_>(),
    })
}

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/join")]
pub async fn join_game(
    game_id: u32,
    user: User,
    games: &State<GameService>,
) -> Result<Json<MessageResponse>, NotFound<Json<MessageResponse>>> {
    if let Some(game) = games.get(game_id) {
        match games.join(game.id, user.id) {
            Ok(_) => Ok(Json(MessageResponse {
                message: "joined the game successfully".into(),
                r#type: "success".into(),
            })),
            Err(e) => Err(NotFound(Json(MessageResponse {
                message: e.into(),
                r#type: "error".into(),
            }))),
        }
    } else {
        Err(NotFound(Json(MessageResponse {
            message: "game not found".into(),
            r#type: "error".into(),
        })))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        responses::GameResponse,
        routes::users::{self as user_routes, tests::create_test_users},
        test::mocked_client,
        users::UserLoginPayload,
    };
    use rocket::http::{ContentType, Status};
    use serde_json;

    #[sqlx::test]
    async fn can_create_game() {
        let client = mocked_client().await;

        create_test_users(&client).await;

        let response = client
            .post(uri!(user_routes::user_login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser1".into(),
                    password: "abc1234".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;

        let game = client
            .post(uri!(super::create_game))
            .private_cookie(response.cookies().get_private("login").unwrap())
            .dispatch()
            .await;

        assert_eq!(game.status(), Status::Created);
        assert!(game.into_json::<GameResponse>().await.is_some());
    }

    #[sqlx::test]
    async fn each_game_gets_individual_ids() {
        let client = mocked_client().await;

        create_test_users(&client).await;

        let response = client
            .post(uri!(user_routes::user_login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser1".into(),
                    password: "abc1234".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;

        let game1 = client
            .post(uri!(super::create_game))
            .private_cookie(response.cookies().get_private("login").unwrap())
            .dispatch()
            .await;
        let game2 = client
            .post(uri!(super::create_game))
            .private_cookie(response.cookies().get_private("login").unwrap())
            .dispatch()
            .await;

        assert_ne!(
            game1.into_json::<GameResponse>().await.unwrap().id,
            game2.into_json::<GameResponse>().await.unwrap().id
        );
    }

    #[sqlx::test]
    async fn can_join_game() {
        let client = mocked_client().await;

        create_test_users(&client).await;

        let response = client
            .post(uri!(user_routes::user_login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser1".into(),
                    password: "abc1234".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;

        let game = client
            .post(uri!(super::create_game))
            .private_cookie(response.cookies().get_private("login").unwrap())
            .dispatch()
            .await;

        let response = client
            .post(uri!(user_routes::user_login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser2".into(),
                    password: "abc1234".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;

        assert_eq!(
            client
                .patch(uri!(super::join_game(
                    game_id = game.into_json::<GameResponse>().await.unwrap().id
                )))
                .private_cookie(response.cookies().get_private("login").unwrap())
                .dispatch()
                .await
                .status(),
            Status::Ok
        );
    }

    #[sqlx::test]
    async fn cannot_join_game_twice() {
        let client = mocked_client().await;

        create_test_users(&client).await;

        let response = client
            .post(uri!(user_routes::user_login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser1".into(),
                    password: "abc1234".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;

        let game = client
            .post(uri!(super::create_game))
            .private_cookie(response.cookies().get_private("login").unwrap())
            .dispatch()
            .await;

        assert_eq!(
            client
                .patch(uri!(super::join_game(
                    game_id = game.into_json::<GameResponse>().await.unwrap().id
                )))
                .private_cookie(response.cookies().get_private("login").unwrap())
                .dispatch()
                .await
                .status(),
            Status::NotFound
        );
    }
}
