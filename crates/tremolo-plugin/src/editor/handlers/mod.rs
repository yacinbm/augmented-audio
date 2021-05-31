use std::sync::Arc;

use tokio::sync::broadcast::{Receiver, Sender};

use ClientMessageInner::{AppStarted, Log, SetParameter};

use crate::editor::list_parameters;
use crate::editor::protocol::{
    ClientMessage, ClientMessageInner, MessageWrapper, PublishParametersMessage, ServerMessage,
    ServerMessageInner,
};
use crate::plugin_parameter::ParameterStore;

pub async fn message_handler_loop(
    mut messages: Receiver<ClientMessage>,
    output_messages: Sender<ServerMessage>,
    parameter_store: &Arc<ParameterStore>,
) {
    loop {
        if let Ok(message) = messages.recv().await {
            match message {
                MessageWrapper { message, .. } => match message {
                    AppStarted(_) => app_started(&output_messages, parameter_store),
                    SetParameter(_) => {}
                    Log(_) => {}
                },
            }
        }
    }
}

fn app_started(
    output_messages: &Sender<MessageWrapper<ServerMessageInner>>,
    parameter_store: &Arc<ParameterStore>,
) {
    let parameters_list = list_parameters(&parameter_store);
    let result = output_messages.send(ServerMessage::notification(
        ServerMessageInner::PublishParameters(PublishParametersMessage {
            parameters: parameters_list,
        }),
    ));

    match result {
        Err(_) => {
            log::error!("Failed to send publish parameters message");
        }
        _ => {}
    }
}
