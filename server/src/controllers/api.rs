use actix::Actor;
use crate::{
    actors::{ClientWsActor, GameActor}, AppState,
    models::messages::ServerCommand, APP_CONFIG,
};
use actix_web::{HttpRequest, Query, Json, State, Path, http::StatusCode};

#[derive(Debug, Deserialize)]
pub struct QueryString {
    key: String,
    name: String,
}
#[derive(Deserialize)]
pub struct RoomOption {
    room_id: i32,
    max_players: i32,
    time: i32,
    // token: String,
}

#[derive(Serialize)]
struct RoomInfo {
    room_id: i32,
}

pub fn socket_handler(
    (req, state, path, query): (HttpRequest<AppState>, State<AppState>, Path<i32>, Query<QueryString>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let room_id = path.into_inner();
    if crate::APP_CONFIG.dev_mode || crate::APP_CONFIG.api_keys.contains(&query.key) {
        if let Some(game_actor_addr) = state.game_addresses.lock().unwrap().get(&room_id) {
            actix_web::ws::start(
                &req,
                ClientWsActor::new(game_actor_addr.clone(), query.key.clone(), query.name.clone()),
            )
        } else {
            Err(actix_web::error::ErrorNotFound("Room not found"))
        }
    } else {
        Err(actix_web::error::ErrorBadRequest("Invalid API Key"))
    }
}

pub fn spectate_handler(
    (req, state, path): (HttpRequest<AppState>, State<AppState>, Path<i32>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let room_id = path.into_inner();
    if let Some(game_actor_addr) = state.game_addresses.lock().unwrap().get(&room_id) {
        actix_web::ws::start(
            &req,
            ClientWsActor::new(
                game_actor_addr.clone(),
                "SPECTATOR".to_string(),
                "SPECTATOR".to_string(),
            ),
        )
    } else {
        Err(actix_web::error::ErrorNotFound("Room not found"))
    }
}

pub fn reset_handler(
    (_req, state, path): (HttpRequest<AppState>, State<AppState>, Path<i32>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let room_id = path.into_inner();
    if let Some(game_actor_addr) = state.game_addresses.lock().unwrap().get(&room_id) {
        // Access the game actor and send a message using do_send
        game_actor_addr.do_send(ServerCommand::Reset);

        Ok(actix_web::HttpResponse::with_body(StatusCode::OK, "done"))
    } else {
        Err(actix_web::error::ErrorNotFound("Room not found"))
    }
}

pub fn list_rooms(
    (_req, state): (HttpRequest<AppState>, State<AppState>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let rooms: Vec<RoomInfo> = state.game_addresses
        .lock()
        .unwrap()
        .keys()
        .map(|key| RoomInfo { room_id: *key })
        .collect();

    Ok(actix_web::HttpResponse::Ok().json(rooms))
}

pub fn create_room(
    (_req, state, body): (HttpRequest<AppState>, State<AppState>, Json<RoomOption>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    // println!("A room created with [ max_players = {}, time = {}, token = {} ]", body.max_players, body.time, body.token);
    let game_actor = GameActor::new(APP_CONFIG.game_config.clone());
    let game_actor_addr = game_actor.start();
    state.game_addresses.lock().unwrap().insert(body.room_id, game_actor_addr);
    Ok(actix_web::HttpResponse::with_body(
        StatusCode::OK, 
        format!("room created with id: {} - max_players: {} - time: {}",  body.room_id, body.max_players, body.time)
    ))
}
