use crate::{int_to_direction, EXPLORER_TOPIC, GROUP_ID, TIMEOUT};
use communication::{
    Communication, Message, PathStatus, PathUnveiledPayload, Payload, TargetPayload,
};
use ev3dev_lang_rust::sound::play;
use robolab_bot_base::communication::BotCommunication;
use robolab_bot_base::planet::Direction;
use std::process::exit;
use std::time::Duration;
pub struct CommunicationHandler {}

impl CommunicationHandler {
    pub(super) fn send_ready(
        c: &mut Box<dyn BotCommunication<M = Message>>,
    ) -> Option<(String, i32, i32, i32)> {
        c.subscribe(EXPLORER_TOPIC)
            .expect("Failed to subscribe to explorer topic");

        if let Err(e) = c.publish(
            EXPLORER_TOPIC,
            CommunicationHandler::message_to_vec(Communication::ready()),
        ) {
            eprintln!("Couldn't send ready: {}", e);
        }

        let message = CommunicationHandler::receive_message(c, TIMEOUT, "ready")?;
        if let Some(Payload::Planet(payload)) = message.payload {
            if let Err(e) = c.unsubscribe(EXPLORER_TOPIC) {
                eprintln!(
                    "Couldn't un subscribe from topic {EXPLORER_TOPIC} in send_ready, {}",
                    e
                );
            }
            Some((
                payload.planet_name,
                payload.start_x,
                payload.start_y,
                payload.start_orientation,
            ))
        } else {
            eprintln!("Ready payload invalid, {:?}", message.payload);
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn send_explored_path(
        c: &mut Box<dyn BotCommunication<M = Message>>,
        planet_name: &str,
        xs: i32,
        ys: i32,
        ds: i32,
        xe: i32,
        ye: i32,
        de: i32,
        status: PathStatus,
    ) -> Option<(i32, i32, i32, i32)> {
        let message = CommunicationHandler::message_to_vec(Communication::explored_path(
            xs, ys, ds, xe, ye, de, status,
        ));
        let topic = format!("planet/{}/{}", planet_name, GROUP_ID);
        if let Err(e) = c.subscribe(topic.as_str()) {
            eprintln!(
                "Couldn't subscribe to topic {topic} in send_explored_path, {}",
                e
            );
        }
        if let Err(e) = c.publish(topic.as_str(), message) {
            eprintln!("Couldn't send explored_path, {}", e);
        }

        let message = CommunicationHandler::receive_message(c, TIMEOUT, "explored_path")?;
        if let Some(Payload::PathServer(payload)) = message.payload {
            println!(
                "Calculated end {}, {}, {} - Real End: {}, {}, {}",
                xe, ye, de, payload.end_x, payload.end_y, payload.end_direction
            );
            if let Err(e) = c.unsubscribe(topic.as_str()) {
                eprintln!(
                    "Couldn't unsubscribe from topic {topic} in send_explored_path, {}",
                    e
                );
            }
            Some((
                payload.end_x,
                payload.end_y,
                payload.end_direction,
                payload.path_weight,
            ))
        } else {
            println!("Invalid explored_path payload");
            None
        }
    }

    pub(super) fn send_select_path(
        c: &mut Box<dyn BotCommunication<M = Message>>,
        x: i32,
        y: i32,
        direction: Direction,
        planet_name: &str,
    ) -> Option<Direction> {
        let topic = format!("planet/{}/{}", planet_name, GROUP_ID);
        if let Err(e) = c.subscribe(topic.as_str()) {
            eprintln!(
                "Couldn't subscribe to topic {topic} in send_select_path, {}",
                e
            );
            return None;
        }
        let path_select_message = CommunicationHandler::message_to_vec(Communication::path_select(
            x,
            y,
            direction as i32,
        ));
        if let Err(e) = c.publish(topic.as_str(), path_select_message) {
            eprintln!("Couldn't send path_select, {}", e);
        }

        let message = CommunicationHandler::receive_message(c, TIMEOUT, "path_select_server")?;
        if let Some(Payload::PathSelectServer(payload)) = message.payload {
            let _ = CommunicationHandler::receive_message(c, TIMEOUT, "path_select_server");
            if let Err(e) = c.unsubscribe(topic.as_str()) {
                eprintln!(
                    "Couldn't unsubscribe from topic {topic} in send_select_path, {}",
                    e
                );
            }
            Some(int_to_direction(payload.start_direction).expect("invalid direction"))
        } else {
            println!("Invalid path select payload");
            None
        }
    }

    pub(super) fn listen_for_paths(
        c: &mut Box<dyn BotCommunication<M = Message>>,
        planet_name: &str,
        timeout: Duration,
    ) -> (Vec<PathUnveiledPayload>, Option<TargetPayload>) {
        let topic = format!("planet/{}/{}", planet_name, GROUP_ID);
        if let Err(e) = c.subscribe(topic.as_str()) {
            eprintln!(
                "Couldn't subscribe to topic {topic} in listen_for_paths, {}",
                e
            );
        }

        let mut paths: Vec<PathUnveiledPayload> = Vec::new();
        let mut target: Option<TargetPayload> = None;

        while let Some(message) =
            CommunicationHandler::receive_message(c, timeout, "listen_for_paths")
        {
            if let Some(Payload::PathUnveiled(payload)) = message.payload {
                paths.push(payload);
            } else if let Some(Payload::Target(payload)) = message.payload {
                target = Some(payload);
            }
        }
        if let Err(e) = c.unsubscribe(topic.as_str()) {
            eprintln!(
                "Couldn't unsubscribe from topic {topic} in listen_for_paths, {}",
                e
            );
        }
        (paths, target)
    }

    pub(super) fn send_target_reached(
        c: &mut Box<dyn BotCommunication<M = Message>>,
        planet_name: &str,
    ) {
        let message = CommunicationHandler::message_to_vec(Communication::target_reached());
        if let Err(e) = c.unsubscribe(format!("planet/{}/{}", planet_name, GROUP_ID).as_str()) {
            eprintln!("Failed to unsubscribe from planet topic {}", e)
        }
        if let Err(e) = c.subscribe(EXPLORER_TOPIC) {
            eprintln!("Failed to subscribe to explorer topic {}", e)
        }

        if let Err(e) = c.publish(EXPLORER_TOPIC, message) {
            eprintln!("Couldn't send target reached, {}", e);
        } else {
            if let Some(message) =
                CommunicationHandler::receive_message(c, TIMEOUT, "target_reached")
            {
                if let Some(Payload::Done(payload)) = message.payload {
                    println!("Target reached acknowledged payload: {}", payload.message);
                    if let Ok(mut sound) = play("./sounds/target-reached/putin.wav") {
                        let _ = sound.wait();
                    }
                    exit(0)
                } else {
                    eprintln!("Failed to parse target_reached payload");
                    exit(-2);
                }
            } else {
                eprintln!("Failed to receive target_reached message");
                exit(-1);
            }
        }
    }

    pub(super) fn send_exploration_completed(
        c: &mut Box<dyn BotCommunication<M = Message>>,
        planet_name: &str,
    ) {
        let message = CommunicationHandler::message_to_vec(Communication::exploration_completed());
        if let Err(e) = c.unsubscribe(format!("planet/{}/{}", planet_name, GROUP_ID).as_str()) {
            eprintln!("Failed to unsubscribe from planet topic {}", e)
        }
        if let Err(e) = c.subscribe(EXPLORER_TOPIC) {
            eprintln!("Failed to subscribe to explorer topic {}", e)
        }

        if let Err(e) = c.publish(EXPLORER_TOPIC, message) {
            eprintln!("Couldn't send exploration completed, {}", e);
        } else {
            if let Some(message) =
                CommunicationHandler::receive_message(c, TIMEOUT, "exploration_completed")
            {
                if let Some(Payload::Done(payload)) = message.payload {
                    if let Ok(mut sound) = play("./sounds/exploration-complete/accordion.wav") {
                        let _ = sound.wait();
                    }
                    println!(
                        "Exploration completed acknowledged payload: {}",
                        payload.message
                    );
                    exit(0)
                } else {
                    eprintln!("Failed to parse exploration_completed payload");
                    exit(-2);
                }
            } else {
                eprintln!("Failed to receive exploration_completed message");
                exit(-1);
            }
        }
    }

    pub(super) fn receive_message(
        c: &mut Box<dyn BotCommunication<M = Message>>,
        timeout: Duration,
        description: &str,
    ) -> Option<Message> {
        if let Ok(Some(publish)) = c.receive_message(timeout)
            && let Ok(Some(message)) = c.parse_message(&publish.topic, &publish.payload)
        {
            if let Ok(mut sound) = play("./sounds/server-path-select/lizard.wav") {
                let _ = sound.wait();
            }
            Some(message)
        } else {
            println!("couldn't receive {description} message");
            None
        }
    }

    #[allow(dead_code)]
    pub(super) fn send_set_planet(c: &mut Box<dyn BotCommunication<M = Message>>, name: &str) {
        if let Err(e) = c.publish(
            EXPLORER_TOPIC,
            CommunicationHandler::message_to_vec(Communication::test_planet(name)),
        ) {
            eprintln!("Couldn't set planet: {}", e);
        }
        let _ = c.receive_message(TIMEOUT); // ignore potential debug response
    }

    pub(super) fn message_to_vec(m: Message) -> Vec<u8> {
        serde_json::to_vec(&m).unwrap()
    }
}
