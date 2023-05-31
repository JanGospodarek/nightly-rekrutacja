use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::Response,
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};

use crate::structs::{
    app_messages::{
        app_messages::ServerToApp, request_rejected::RequestRejected,
        sign_messages::SignMessagesResponse, sign_transactions::SignTransactionsResponse,
        user_connected_event::UserConnectedEvent, user_disconnected_event::UserDisconnectedEvent,
    },
    client_messages::{
        client_messages::{ClientToServer, ServerToClient},
        connect::ConnectResponse,
        get_info::GetInfoResponse,
        get_pending_requests::GetPendingRequestsResponse,
    },
    common::{AckMessage, SessionStatus},
    pending_request::PendingRequest,
    session::Session,
};

pub async fn on_new_client_connection(
    ConnectInfo(ip): ConnectInfo<SocketAddr>,
    State(sessions): State<Arc<DashMap<String, Session>>>,
    ws: WebSocketUpgrade,
) -> Response {
    println!("New client connection from {}", ip);
    ws.on_upgrade(move |socket| client_handler(socket, sessions))
}

pub async fn client_handler(socket: WebSocket, sessions: Arc<DashMap<String, Session>>) {
    let (mut sender, mut receiver) = socket.split();
    // Handle the new app connection here
    // Wait for initialize message
    let session_id = loop {
        let sessions = sessions.clone();
        let msg = match receiver.next().await {
            Some(msg) => match msg {
                Ok(msg) => msg,
                Err(_e) => {
                    return;
                }
            },
            None => {
                return;
            }
        };
        println!("Message received {:?}", msg);
        let app_msg = match msg {
            Message::Text(data) => match serde_json::from_str::<ClientToServer>(&data) {
                Ok(app_msg) => app_msg,
                Err(_) => continue,
            },
            Message::Binary(_) => continue,
            Message::Close(None) | Message::Close(Some(_)) => {
                return;
            }
            Message::Ping(_) => {
                continue;
            }
            Message::Pong(_) => {
                continue;
            }
        };
        println!("Message received {:?}", app_msg);

        match app_msg {
            ClientToServer::GetInfoRequest(get_info_request) => {
                println!("Get info request received {} ", get_info_request.session_id);
                let session = sessions.get(&get_info_request.session_id).unwrap();
                let response = ServerToClient::GetInfoResponse(GetInfoResponse {
                    response_id: get_info_request.response_id,
                    network: session.network.clone(),
                    version: session.version.clone(),
                    app_metadata: session.app_state.metadata.clone(),
                });
                sender
                    .send(Message::Text(serde_json::to_string(&response).unwrap()))
                    .await
                    .unwrap()
            }
            ClientToServer::ConnectRequest(connect_request) => {
                println!("Connect request received {} ", connect_request.session_id);
                let mut session = sessions.get_mut(&connect_request.session_id).unwrap();
                // Insert user socket
                session.update_status(SessionStatus::ClientConnected);
                session.client_state.client_socket = Some(sender);
                session.client_state.device = connect_request.device.clone();
                session.client_state.connected_public_keys = connect_request.public_keys.clone();

                let client_reponse = ServerToClient::ConnectResponse(ConnectResponse {
                    response_id: connect_request.response_id,
                });
                session.send_to_client(client_reponse).await.unwrap();
                let app_event = ServerToApp::UserConnectedEvent(UserConnectedEvent {
                    public_keys: connect_request.public_keys,
                });
                session.send_to_app(app_event).await.unwrap();

                break session.session_id.clone();
            }
            _ => {
                continue;
            }
        }
    };
    // Main loop request handler
    loop {
        let sessions = sessions.clone();
        let msg = match receiver.next().await {
            Some(msg) => match msg {
                Ok(msg) => msg,
                Err(_e) => {
                    let user_disconnected_event =
                        ServerToApp::UserDisconnectedEvent(UserDisconnectedEvent {});
                    let mut session = match sessions.get_mut(&session_id) {
                        Some(session) => session,
                        None => return,
                    };
                    session
                        .send_to_app(user_disconnected_event)
                        .await
                        .unwrap_or_default();
                    session.update_status(SessionStatus::UserDisconnected);

                    return;
                }
            },
            None => {
                let user_disconnected_event =
                    ServerToApp::UserDisconnectedEvent(UserDisconnectedEvent {});
                let mut session = match sessions.get_mut(&session_id) {
                    Some(session) => session,
                    None => return,
                };
                session
                    .send_to_app(user_disconnected_event)
                    .await
                    .unwrap_or_default();
                session.update_status(SessionStatus::UserDisconnected);

                return;
            }
        };
        let app_msg = match msg {
            Message::Text(data) => match serde_json::from_str::<ClientToServer>(&data) {
                Ok(app_msg) => app_msg,
                Err(_) => continue,
            },
            Message::Binary(_) => continue,
            Message::Close(None) | Message::Close(Some(_)) => {
                let user_disconnected_event =
                    ServerToApp::UserDisconnectedEvent(UserDisconnectedEvent {});
                let mut session = match sessions.get_mut(&session_id) {
                    Some(session) => session,
                    None => return,
                };
                session
                    .send_to_app(user_disconnected_event)
                    .await
                    .unwrap_or_default();
                session.update_status(SessionStatus::UserDisconnected);

                return;
            }
            Message::Ping(_) => {
                continue;
            }
            Message::Pong(_) => {
                continue;
            }
        };
        match app_msg {
            ClientToServer::SignTransactionsEventReply(sign_transactions_event_reply) => {
                let mut session = sessions.get_mut(&session_id).unwrap();
                let _pending_request = session
                    .pending_requests
                    .remove(&sign_transactions_event_reply.request_id)
                    .unwrap();
                // Send to app
                let app_msg = ServerToApp::SignTransactionsResponse(SignTransactionsResponse {
                    response_id: sign_transactions_event_reply.request_id.clone(),
                    signed_transactions: sign_transactions_event_reply.signed_transactions,
                    metadata: sign_transactions_event_reply.metadata,
                });
                session.send_to_app(app_msg).await.unwrap();

                let client_msg = ServerToClient::AckMessage(AckMessage {
                    response_id: sign_transactions_event_reply.response_id,
                });
                session.send_to_client(client_msg).await.unwrap();
            }
            ClientToServer::SignMessagesEventReply(sign_message_event_reply) => {
                let mut session = sessions.get_mut(&session_id).unwrap();
                let _pending_request = session
                    .pending_requests
                    .remove(&sign_message_event_reply.request_id)
                    .unwrap();
                // Send to app
                let app_msg = ServerToApp::SignMessagesResponse(SignMessagesResponse {
                    response_id: sign_message_event_reply.request_id.clone(),
                    signed_messages: sign_message_event_reply.signed_messages,
                    metadata: sign_message_event_reply.metadata,
                });
                session.send_to_app(app_msg).await.unwrap();

                let client_msg = ServerToClient::AckMessage(AckMessage {
                    response_id: sign_message_event_reply.response_id,
                });
                session.send_to_client(client_msg).await.unwrap();
            }
            ClientToServer::GetInfoRequest(get_info_request) => {
                let mut session = sessions.get_mut(&get_info_request.session_id).unwrap();
                let response = ServerToClient::GetInfoResponse(GetInfoResponse {
                    response_id: get_info_request.response_id,
                    network: session.network.clone(),
                    version: session.version.clone(),
                    app_metadata: session.app_state.metadata.clone(),
                });
                session.send_to_client(response).await.unwrap();
            }
            ClientToServer::GetPendingRequestsRequest(get_pending_requests_request) => {
                let mut session = sessions.get_mut(&session_id).unwrap();
                let pending_requests = session
                    .pending_requests
                    .clone()
                    .iter()
                    .map(|v| v.clone())
                    .collect::<Vec<PendingRequest>>();
                let response =
                    ServerToClient::GetPendingRequestsResponse(GetPendingRequestsResponse {
                        requests: pending_requests,
                        response_id: get_pending_requests_request.response_id,
                    });
                session.send_to_client(response).await.unwrap();
            }
            ClientToServer::Reject(reject) => {
                let mut session = sessions.get_mut(&session_id).unwrap();
                session.pending_requests.remove(&reject.request_id);
                // Send to app
                let app_msg = ServerToApp::RequestRejected(RequestRejected {
                    response_id: reject.request_id.clone(),
                    reason: reject.reason,
                });
                session.send_to_app(app_msg).await.unwrap();

                let client_msg = ServerToClient::AckMessage(AckMessage {
                    response_id: reject.response_id,
                });
                session.send_to_client(client_msg).await.unwrap();
            }
            _ => {
                continue;
            }
        }
    }
}
