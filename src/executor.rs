use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio_tungstenite::tungstenite::Message as TokioMessage;

use futures::{SinkExt, StreamExt, TryStreamExt};
use reqwest::Client;
use tokio::{
    select,
    sync::mpsc::{channel, Receiver, Sender},
    time::sleep,
};
use tokio_tungstenite::connect_async;

use crate::{
    settings::{Method, Protocol},
    states::main_page::{Header, RequestData, Response},
};

#[derive(PartialEq, Clone, Debug)]
pub enum State {
    FREE,
    BUSY,
    CONNECTED,
}

#[derive(Debug)]
pub enum Message {
    COMMAND(Command),
    RESULT(Result),
}

impl Message {
    pub fn execute(data: &RequestData) -> Self {
        Message::COMMAND(data.into())
    }

    pub fn terminate() -> Self {
        Message::COMMAND(Command::termiate())
    }

    pub fn get_command(self) -> Option<Command> {
        match self {
            Message::COMMAND(command) => Some(command),
            Message::RESULT(_) => None,
        }
    }

    pub fn get_result(self) -> Option<Result> {
        match self {
            Message::COMMAND(_) => None,
            Message::RESULT(result) => Some(result),
        }
    }
}

impl From<&RequestData> for Message {
    fn from(value: &RequestData) -> Self {
        Message::COMMAND(value.into())
    }
}

#[derive(Debug)]
pub enum Command {
    EXECUTE(CommandExecute),
    TERRMINATE,
}

impl From<&RequestData> for Command {
    fn from(value: &RequestData) -> Self {
        Self::EXECUTE(value.into())
    }
}

impl Command {
    pub fn execute(data: &RequestData) -> Self {
        Self::EXECUTE(data.into())
    }

    pub fn termiate() -> Self {
        Self::TERRMINATE
    }
}

#[derive(Debug)]
pub struct Result {}

#[derive(Debug)]
pub struct CommandExecute {
    pub uri: String,
    pub protocol: Protocol,
    pub method: Method,
    pub headers: Vec<Header>,
    pub body: String,
}

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
            body: value.body.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Executor {
    pub state: Arc<Mutex<State>>,
    responses: Arc<Mutex<Vec<Response>>>,
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

    pub fn execute(&mut self, data: &RequestData) {
        let message: Message = data.into();

        let state = match self.state.lock() {
            Ok(state) => (*state).clone(),
            Err(err) => {
                println!("Error: Could not get mutex of Executor State: {err}");
                return;
            }
        };

        if state == State::CONNECTED {
            self.send_ws(message);
            return;
        }

        if state == State::FREE
            && (data.protocol == Protocol::HTTP || data.protocol == Protocol::HTTPS)
        {
            *self.state.lock().unwrap() = State::BUSY;
            if let Some(command) = message.get_command() {
                self.execute_http(command);
            }

            return;
        }

        if state == State::FREE && (data.protocol == Protocol::WS || data.protocol == Protocol::WSS)
        {
            *self.state.lock().unwrap() = State::CONNECTED;
            if let Some(command) = message.get_command() {
                self.connect_ws(command);
            }

            return;
        }
    }
    pub fn terminate(&mut self) {
        if self.channel_sender.is_none() {
            *self.state.lock().unwrap() = State::FREE;
            return;
        };
        let _ = self
            .channel_sender
            .as_mut()
            .unwrap()
            .send(Message::terminate());
        *self.state.lock().unwrap() = State::FREE;
    }

    fn execute_http(&mut self, message: Command) {
        let (sender, receiver) = channel::<Message>(100);
        self.channel_sender = Some(sender);
        let responses = Arc::clone(&self.responses);
        tokio::spawn(Self::http_thread(
            message,
            responses,
            receiver,
            self.state.clone(),
        ));
    }

    async fn http_thread(
        message: Command,
        responses: Arc<Mutex<Vec<Response>>>,
        mut command_channel: Receiver<Message>,
        executor_state: Arc<Mutex<State>>,
    ) {
        match message {
            Command::EXECUTE(command_execute) => {
                let request_future = async {
                    let uri = format!("{}://{}", command_execute.protocol, command_execute.uri);
                    let result = Client::new()
                        .request(command_execute.method.into(), uri.clone())
                        .send()
                        .await;
                    let response = match result {
                        Ok(val) => Response::from_http_response(val).await,
                        Err(err) => Response::from_http_error(err).await,
                    };

                    match responses.lock() {
                        Ok(mut r) => {
                            r.push(response);
                            return;
                        }
                        Err(_err) => return,
                    };
                };

                let terminate_future = async {
                    loop {
                        // любая команда сюда приводит к ответе текущего выполнения
                        command_channel.recv().await;
                    }
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

    fn connect_ws(&mut self, message: Command) {
        let (sender, receiver) = channel::<Message>(100);
        self.channel_sender = Some(sender);

        tokio::spawn(Self::ws_thread(
            message,
            Arc::clone(&self.responses),
            receiver,
            self.state.clone(),
        ));
    }

    async fn ws_thread(
        command: Command,
        responses: Arc<Mutex<Vec<Response>>>,
        mut command_channel: Receiver<Message>,
        executor_state: Arc<Mutex<State>>,
    ) {
        match command {
            Command::EXECUTE(command_execute) => {
                let uri = format!("{}://{}", command_execute.protocol, command_execute.uri);

                let (ws_stream, _) = match connect_async(&uri).await {
                    Ok(data) => data,
                    Err(err) => {
                        println!("Error: Could not connect to WS. Error: {err}");
                        *executor_state.lock().unwrap() = State::FREE;
                        return;
                    }
                };

                let (mut write, mut read) = ws_stream.split();

                loop {
                    select! {
                        message = read.try_next() => {
                            match message {
                                Ok(Some(val)) => {
                                    match val {
                                        TokioMessage::Text(utf8_bytes) => {
                                            responses.lock().unwrap().push(
                                                Response::from_utf8_bytes(utf8_bytes)
                                            );
                                            continue;
                                        },
                                        TokioMessage::Close(_) => {
                                            println!("Peer close connection.");
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
                                    println!("Error: During WS connection on new message error occured. Error: {e}");
                                    *executor_state.lock().unwrap() = State::FREE;
                                    break;
                                }
                            };
                        }
                        _ = sleep(Duration::from_millis(50)) => {
                            let message = command_channel.recv().await.unwrap();

                            match message {
                                Message::COMMAND(command) => match command {
                                    Command::EXECUTE(command_execute) => {
                                        let result = write
                                            .send(tokio_tungstenite::tungstenite::Message::text(
                                                command_execute.body.clone(),
                                            ))
                                            .await;

                                        match result {
                                            Ok(_) => {}
                                            Err(err) => {
                                                println!("Error: Could not send message to WS channel. Message: {}. Error: {}",command_execute.body.clone(), err);
                                            }
                                        };
                                    }
                                    Command::TERRMINATE => {
                                        println!("Requested WS termination.");
                                        *executor_state.lock().unwrap() = State::FREE;
                                        break;
                                    }
                                },
                                Message::RESULT(result) => {
                                    println!("Error: Received in command channel Result type: {result:#?}");
                                }
                            };
                        }

                    }
                }
            }
            Command::TERRMINATE => {
                *executor_state.lock().unwrap() = State::FREE;
                return;
            }
        }
    }

    fn send_ws(&mut self, message: Message) {
        if self.channel_sender.is_none() {
            *self.state.lock().unwrap() = State::FREE;
            return;
        };
        let _ = self.channel_sender.as_mut().unwrap().send(message);
    }
}
