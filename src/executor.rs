/// Executro engine for reqeusting or keep sessions for requests
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio_tungstenite::tungstenite::Message as TokioMessage;

use futures::{executor::block_on, SinkExt, StreamExt, TryStreamExt};
use reqwest::Client;
use tokio::{
    select,
    sync::mpsc::{channel, Receiver, Sender},
    time,
};
use tokio_tungstenite::connect_async;

use crate::{
    settings::{Method, Protocol},
    states::{
        main_page::{generics::Header, request::request_data::RequestData, response::Response},
        Events,
    },
};

/// Executor stares
#[derive(PartialEq, Clone, Debug)]
pub enum State {
    FREE,
    BUSY,
    CONNECTED,
}

/// Message for queue between main thread and executor thread
#[derive(Debug)]
pub enum Message {
    /// command for executor to execute
    COMMAND(Command),
    /// executor response
    /// usualy executor dont respond and wrinting directly to responses vec
    RESULT(Result),
}

impl Message {
    /// Execute command with payload
    pub fn execute(data: &RequestData) -> Self {
        Message::COMMAND(data.into())
    }

    /// Terminate currently executing process
    pub fn terminate() -> Self {
        Message::COMMAND(Command::termiate())
    }

    /// Get Self as Command
    pub fn get_command(self) -> Option<Command> {
        match self {
            Message::COMMAND(command) => Some(command),
            Message::RESULT(_) => None,
        }
    }

    /// Get Self as Result
    pub fn get_result(self) -> Option<Result> {
        match self {
            Message::COMMAND(_) => None,
            Message::RESULT(result) => Some(result),
        }
    }
}

/// From ReqeustData payload -> queue message
impl From<&RequestData> for Message {
    fn from(value: &RequestData) -> Self {
        Message::COMMAND(value.into())
    }
}

/// Command to execute on executor
#[derive(Debug)]
pub enum Command {
    /// make reqeust with payload
    EXECUTE(CommandExecute),
    /// terminate currently pending job
    TERRMINATE,
}

/// from ReqeustData payload -> command to execute
impl From<&RequestData> for Command {
    fn from(value: &RequestData) -> Self {
        Self::EXECUTE(value.into())
    }
}

impl Command {
    /// Get Command as execute
    pub fn execute(data: &RequestData) -> Self {
        Self::EXECUTE(data.into())
    }

    // Get Command as termiate
    pub fn termiate() -> Self {
        Self::TERRMINATE
    }

    pub fn drop_message(&mut self) {
        match self {
            Command::EXECUTE(command_execute) => command_execute.message = "".into(),
            Command::TERRMINATE => {}
        }
    }
}

/// Executor result
/// Currently dont use
#[derive(Debug)]
pub struct Result {}

/// Command to execute on executor
#[derive(Debug)]
pub struct CommandExecute {
    pub uri: String,
    pub protocol: Protocol,
    pub method: Method,
    pub headers: Vec<Header>,
    pub body: String,
    pub message: String,
}

/// From RequestData -> command to execute on executor
impl From<&RequestData> for CommandExecute {
    fn from(value: &RequestData) -> Self {
        let headers = value
            .headers
            .iter()
            .map(|f| Header {
                key: f.key.clone(),
                value: f.value.clone(),
            })
            .collect();
        Self {
            uri: value.uri.clone(),
            protocol: value.protocol.clone(),
            method: value.method.clone(),
            headers,
            body: value.body.message.clone(),
            message: value.message.message.clone(),
        }
    }
}

// TODO: add events and errors or warn on bad responses

/// Executor engine
#[derive(Debug, Clone)]
pub struct Executor {
    /// current executor state
    pub state: Arc<Mutex<State>>,
    /// vec or responses for current reqeust
    responses: Arc<Mutex<Vec<Response>>>,
    /// channel to throw commands from main thread to executor thread
    channel_sender: Option<Sender<Message>>,
}

impl Executor {
    pub fn new(responses: Arc<Mutex<Vec<Response>>>) -> Self {
        Self {
            state: Arc::new(Mutex::new(State::FREE)),
            responses,
            channel_sender: None,
        }
    }

    /// execute action based on payload.
    /// data - request data.
    /// connection_only - mean we need initiate only connection, without sending 1st message.
    ///     Ignored when WS already connected or when protocot is HTTP-like.
    pub fn execute(
        &mut self,
        data: &RequestData,
        connection_only: bool,
        events: Arc<Mutex<Events>>,
    ) {
        let message: Message = data.into();

        let state = match self.state.lock() {
            Ok(state) => (*state).clone(),
            Err(err) => {
                events.lock().unwrap().event_error(&format!(
                    "Error: Could not get mutex of Executor State: {err}"
                ));
                return;
            }
        };

        // If executor connected == currently websocket session going on
        // Passing message to session
        // In usual way you cant pass data with protocol other than Websocket without termination connection
        // This must be guaranted by UI
        if state == State::CONNECTED {
            events.lock().unwrap().event_info(&format!(
                "Detected connected websocket session, sending request..."
            ));
            self.send_ws(message);
            return;
        }

        // If executor is free and requested usual HTTP request
        if state == State::FREE
            && (data.protocol == Protocol::HTTP || data.protocol == Protocol::HTTPS)
        {
            events
                .lock()
                .unwrap()
                .event_info(&format!("Detected free executor, sending http request..."));
            *self.state.lock().unwrap() = State::BUSY;
            if let Some(command) = message.get_command() {
                self.spawn_http_connection(command, events);
            }

            return;
        }

        // if executor is free and requested usual Weebsocket connecrtion
        if state == State::FREE && (data.protocol == Protocol::WS || data.protocol == Protocol::WSS)
        {
            events.lock().unwrap().event_info(&format!(
                "Detected free executor, sending websocket request..."
            ));
            *self.state.lock().unwrap() = State::CONNECTED;
            if let Some(mut command) = message.get_command() {
                // If we need only connection - removing data from command, so it wont be executed during connection initialization
                if connection_only {
                    command.drop_message();
                };

                self.spawn_ws_connection(command, events);
            }

            return;
        }
    }

    /// Terminate currently pending requests, if any
    pub async fn terminate(&mut self) {
        if self.channel_sender.is_none() {
            *self.state.lock().unwrap() = State::FREE;
            return;
        };

        let _ = self
            .channel_sender
            .as_mut()
            .unwrap()
            .send(Message::terminate())
            .await;
        // *self.state.lock().unwrap() = State::FREE;
    }

    /// Spawn separate thread for http reqeusts
    fn spawn_http_connection(&mut self, message: Command, events: Arc<Mutex<Events>>) {
        let (sender, receiver) = channel::<Message>(100);
        self.channel_sender = Some(sender);

        tokio::spawn(Self::http_thread(
            message,
            Arc::clone(&self.responses),
            Arc::clone(&events),
            receiver,
            self.state.clone(),
        ));
    }

    /// Thread for http requests
    async fn http_thread(
        message: Command,
        responses: Arc<Mutex<Vec<Response>>>,
        events: Arc<Mutex<Events>>,
        mut command_channel: Receiver<Message>,
        executor_state: Arc<Mutex<State>>,
    ) {
        match message {
            Command::EXECUTE(command_execute) => {
                let request_future = async {
                    let uri = format!("{}://{}", command_execute.protocol, command_execute.uri);

                    // sleep(Duration::from_secs(50)).await;

                    let result = Client::new()
                        .request(command_execute.method.into(), uri.clone())
                        .send()
                        .await;
                    let response = match result {
                        Ok(val) => match Response::from_http_response(val).await {
                            Ok(r) => {
                                events
                                    .lock()
                                    .unwrap()
                                    .event_info(&format!("Received success response"));
                                r
                            }
                            Err((r, err)) => {
                                events.lock().unwrap().event_error(&err);
                                r
                            }
                        },
                        Err(err) => match Response::from_http_error(err).await {
                            Ok(r) => {
                                events
                                    .lock()
                                    .unwrap()
                                    .event_info(&format!("Received success error response"));
                                r
                            }
                            Err((r, err)) => {
                                events.lock().unwrap().event_error(&err);
                                r
                            }
                        },
                    };

                    match responses.lock() {
                        Ok(mut r) => {
                            r.push(response);
                            return;
                        }
                        Err(_) => {
                            return;
                        }
                    };
                };

                let terminate_future = async {
                    // любая команда сюда приводит к ответе текущего выполнения
                    command_channel.recv().await;
                };

                select! {
                    _ = request_future => {
                        *executor_state.lock().unwrap() = State::FREE;
                    }
                    _ = terminate_future => {
                        *executor_state.lock().unwrap() = State::FREE;
                    }
                };
            }
            Command::TERRMINATE => {
                *executor_state.lock().unwrap() = State::FREE;
                return;
            }
        }
    }

    /// Spawn separate thread for websocket
    fn spawn_ws_connection(&mut self, command: Command, events: Arc<Mutex<Events>>) {
        let (sender, receiver) = channel::<Message>(100);
        self.channel_sender = Some(sender);

        tokio::spawn(Self::ws_thread(
            command,
            Arc::clone(&self.responses),
            Arc::clone(&events),
            receiver,
            self.state.clone(),
        ));
    }

    /// Thread for websocket requests
    async fn ws_thread(
        command: Command,
        responses: Arc<Mutex<Vec<Response>>>,
        events: Arc<Mutex<Events>>,
        mut command_channel: Receiver<Message>,
        executor_state: Arc<Mutex<State>>,
    ) {
        let _ = events;
        match command {
            Command::EXECUTE(command_execute) => {
                let uri = format!(
                    "{}://{}",
                    command_execute.protocol.to_string().to_lowercase(),
                    command_execute.uri
                );

                let (ws_stream, _) = match connect_async(&uri).await {
                    Ok(data) => data,
                    Err(err) => {
                        events
                            .lock()
                            .unwrap()
                            .event_error(&format!("Error: Could not connect to WS. Error: {err}"));

                        *executor_state.lock().unwrap() = State::FREE;
                        return;
                    }
                };

                let mut request_interval = time::interval(Duration::from_millis(60));

                let (mut write, mut read) = ws_stream.split();

                events
                    .lock()
                    .unwrap()
                    .event_info(&"Websocket: start messages loop...".into());

                // if have some initial command on ws connection sending it
                if command_execute.message.len() > 0 {
                    let _ = write
                        .send(tokio_tungstenite::tungstenite::Message::text(
                            command_execute.message.clone(),
                        ))
                        .await;
                }

                loop {
                    select! {
                        message = read.try_next() => {
                            match message {
                                Ok(Some(val)) => {
                                    match val {
                                        TokioMessage::Text(utf8_bytes) => {
                                            events
                                                .lock()
                                                .unwrap()
                                                .event_info(&format!("Websocket: received response"));
                                            responses.lock().unwrap().push(
                                                Response::from_utf8_bytes(utf8_bytes)
                                            );
                                            continue;
                                        },
                                        TokioMessage::Close(_) => {
                                            events
                                                .lock()
                                                .unwrap()
                                                .event_warning(&"Peer close connection.".into());

                                            responses.lock().unwrap().push(
                                                Response::closed_connection()
                                            );
                                            *executor_state.lock().unwrap() = State::FREE;
                                            break;
                                        },
                                        _ => {
                                            continue;
                                        }
                                    }
                                },
                                Ok(None) => {continue;}
                                Err(e) =>{
                                    events
                                        .lock()
                                        .unwrap()
                                        .event_error(&format!("Error: During WS connection on new message error occured. Error: {e}"));
                                    responses.lock().unwrap().push(
                                        Response::closed_connection()
                                    );
                                    *executor_state.lock().unwrap() = State::FREE;
                                    break;
                                }
                            };
                        }

                        // Checking requests queue to send
                        _ = request_interval.tick() => {

                            let channel_event = command_channel.try_recv();

                            let message = match channel_event {
                                Ok(val) => val,
                                Err(_) => continue,
                            };

                            match message {
                                Message::COMMAND(command) => match command {
                                    Command::EXECUTE(command_execute) => {
                                        events
                                            .lock()
                                            .unwrap()
                                            .event_info(&format!("Websocket: sending message"));
                                        let result = write
                                            .send(tokio_tungstenite::tungstenite::Message::text(
                                                command_execute.message.clone(),
                                            ))
                                            .await;

                                        match result {
                                            Ok(_) => {}
                                            Err(err) => {
                                                events
                                                    .lock()
                                                    .unwrap()
                                                    .event_error(&format!("Error: Could not send message to WS channel. Message: {}. Error: {}",command_execute.body.clone(), err));
                                            }
                                        };
                                    }
                                    Command::TERRMINATE => {
                                        events
                                            .lock()
                                            .unwrap()
                                            .event_info(&"Websocket: requested WS termination.".into());

                                        *executor_state.lock().unwrap() = State::FREE;
                                        break;
                                    }
                                },
                                Message::RESULT(result) => {
                                    events
                                        .lock()
                                        .unwrap()
                                        .event_warning(&format!("Error: Received in command channel Result type: {result:#?}"));

                                }
                            };
                        }

                    }
                }
            }
            Command::TERRMINATE => {
                events
                    .lock()
                    .unwrap()
                    .event_info(&"Requested WS termination.".into());

                *executor_state.lock().unwrap() = State::FREE;
                return;
            }
        }
    }

    /// Send command to queue
    fn send_ws(&mut self, message: Message) {
        if self.channel_sender.is_none() {
            *self.state.lock().unwrap() = State::FREE;
            return;
        };
        let _ = block_on(self.channel_sender.as_mut().unwrap().send(message));
    }
}
