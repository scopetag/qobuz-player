use assets::static_handler;
use axum::{
    extract::State,
    response::{sse::Event, Sse},
    routing::get,
    Router,
};
use futures::stream::Stream;
use leptos::html::*;
use leptos::*;
use qobuz_player_controls::notification::Notification;
use routes::{album, artist, discover, favorites, now_playing, playlist, queue, search};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::broadcast::{self, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

mod assets;
mod components;
mod icons;
mod page;
mod routes;
mod view;

pub fn is_htmx_request(headers: &axum::http::HeaderMap) -> bool {
    headers.get("HX-Request").is_some() && headers.get("HX-Boosted").is_none()
}

pub async fn init(address: String) {
    println!("Listening on {address}");
    let router = create_router().await;
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let mut broadcast_receiver = qobuz_player_controls::notify_receiver();

            loop {
                if let Ok(message) = broadcast_receiver.recv().await {
                    if message == Notification::Quit {
                        break;
                    }
                }
            }
        })
        .await
        .unwrap();
}

async fn create_router() -> Router {
    let (tx, _rx) = broadcast::channel::<ServerSentEvent>(100);
    let shared_state = Arc::new(AppState { tx: tx.clone() });
    tokio::spawn(background_task(tx));

    axum::Router::new()
        .route("/sse", get(sse_handler))
        .with_state(shared_state)
        .merge(now_playing::routes())
        .merge(search::routes())
        .merge(album::routes())
        .merge(artist::routes())
        .merge(playlist::routes())
        .merge(favorites::routes())
        .merge(queue::routes())
        .merge(discover::routes())
        .route("/assets/{*file}", get(static_handler))
}

async fn background_task(tx: Sender<ServerSentEvent>) {
    let mut receiver = qobuz_player_controls::notify_receiver();

    loop {
        if let Ok(notification) = receiver.recv().await {
            match notification {
                Notification::Status { status } => {
                    let message_data = match status {
                        qobuz_player_controls::State::VoidPending => "pause",
                        qobuz_player_controls::State::Null => "pause",
                        qobuz_player_controls::State::Ready => "pause",
                        qobuz_player_controls::State::Paused => "pause",
                        qobuz_player_controls::State::Playing => "play",
                    };

                    let event = ServerSentEvent {
                        event_name: "status".into(),
                        event_data: message_data.into(),
                    };
                    _ = tx.send(event);
                }
                Notification::Position { clock } => {
                    let event = ServerSentEvent {
                        event_name: "position".into(),
                        event_data: clock.seconds().to_string(),
                    };
                    _ = tx.send(event);
                }
                Notification::CurrentTrackList { list: _ } => {
                    let event = ServerSentEvent {
                        event_name: "tracklist".into(),
                        event_data: Default::default(),
                    };
                    _ = tx.send(event);
                }
                Notification::Quit => (),
                Notification::Error { error: _ } => (),
                Notification::Volume { volume } => {
                    let event = ServerSentEvent {
                        event_name: "volume".into(),
                        event_data: volume.to_string(),
                    };
                    _ = tx.send(event);
                }
            };
        }
    }
}

async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => Some(Ok(Event::default()
            .event(event.event_name)
            .data(event.event_data))),
        Err(_) => None,
    });

    Sse::new(stream)
}

pub struct AppState {
    pub tx: Sender<ServerSentEvent>,
}

#[derive(Clone)]
pub struct ServerSentEvent {
    event_name: String,
    event_data: String,
}
