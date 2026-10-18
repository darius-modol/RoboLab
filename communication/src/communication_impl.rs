#![allow(dead_code)]

use std::time::Duration;

use robolab_bot_base::{communication::BotCommunication, RlCommunication};
use rumqttc as mqtt;
use rumqttc::{Event, Incoming, Outgoing, QoS};
use serde::{Deserialize, Serialize};

/// Flattened error type for rumqttc errors.
#[derive(Debug)]
pub enum EasyError {
    Client(mqtt::ClientError),
    Connection(mqtt::ConnectionError),
    Recv(mqtt::RecvError),
    RecvTimeout(mqtt::RecvTimeoutError),
}

impl core::fmt::Display for EasyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl core::error::Error for EasyError {}

/// Class to hold the MQTT client communication
/// Feel Free to add functions, remove fields, or update the constructor to
/// satisfy the requirements and thereby solve the task according to the specifications
#[derive(RlCommunication)]
pub struct Communication {
    client: mqtt::Client,
    connection: mqtt::Connection,
}

// WARN: You can edit this message type however you want but **the type name and visibility MUST NOT be changed**.
//
// Your Message type is part of the public interface of this library.
// payload als Option, da nicht alle Nachrichten Payload besitzen

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Message {
    pub from: From,
    #[serde(rename = "type")]
    pub typ: Typ,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Payload>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum From {
    Client,
    Server,
    Debug,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Typ {
    Ready,
    TestPlanet,
    Planet,
    PathSelect,
    Path,
    PathUnveiled,
    Target,
    TargetReached,
    ExplorationCompleted,
    Done,
    Notice,
    Syntax,
    Error,
}

///untagged, sorgt für zuordnung anhand der Struktur (vorhandene Felder des Jsons)
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Payload {
    //TestPlanet und Ready Payloads des Servers
    TestPlanetPayload(TestPlanetPayload),
    Planet(PlanetPayload),

    PathSelectClient(PathSelectClientPayload),
    PathSelectServer(PathSelectServerPayload),

    PathClient(PathClientPayload),
    PathServer(PathServerPayload),

    PathUnveiled(PathUnveiledPayload),
    Target(TargetPayload),

    TargetReached(TargetReachedPayload),
    ExplorationCompleted(ExplorationCompletedPayload),

    Done(DonePayload),
    Notice(NoticePayload),
    Syntax(SyntaxPayload),
    Error(ErrorPayload),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PathStatus {
    Free,
    Blocked,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TestPlanetPayload {
    pub planet_name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlanetPayload {
    pub planet_name: String,
    pub start_x: i32,
    pub start_y: i32,
    pub start_orientation: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PathSelectClientPayload {
    pub start_x: i32,
    pub start_y: i32,
    pub start_direction: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PathSelectServerPayload {
    pub start_direction: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PathClientPayload {
    pub start_x: i32,
    pub start_y: i32,
    pub start_direction: i32,
    pub end_x: i32,
    pub end_y: i32,
    pub end_direction: i32,
    pub path_status: PathStatus,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PathServerPayload {
    pub start_x: i32,
    pub start_y: i32,
    pub start_direction: i32,
    pub end_x: i32,
    pub end_y: i32,
    pub end_direction: i32,
    pub path_status: PathStatus,
    pub path_weight: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PathUnveiledPayload {
    pub start_x: i32,
    pub start_y: i32,
    pub start_direction: i32,
    pub end_x: i32,
    pub end_y: i32,
    pub end_direction: i32,
    pub path_status: PathStatus,
    pub path_weight: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TargetPayload {
    pub target_x: i32,
    pub target_y: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TargetReachedPayload {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExplorationCompletedPayload {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DonePayload {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NoticePayload {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ErrorPayload {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SyntaxPayload {
    pub message: String,
    pub errors: Vec<String>,
}

impl BotCommunication for Communication {
    type M = Message;
    ///loopt bis Nachrichten vom Server oder von Debug ankommen. Client Nachrichten und Pings werden ignoriert
    fn receive_message(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<mqtt::Publish>, Box<dyn core::error::Error>> {
        use mqtt::{Event, Incoming, Outgoing};
        use tracing::*;

        loop {
            let received_data = self
                .connection
                .recv_timeout(timeout)
                .map_err(EasyError::RecvTimeout)?;
            let event = received_data.map_err(EasyError::Connection)?;

            match event {
                Event::Incoming(Incoming::Publish(mqtt_msg)) => {
                    trace!("<raw {mqtt_msg:?}");
                    if let Some(message) = self.parse_message("yey", &mqtt_msg.payload)? {
                        match message.from {
                            From::Client => {
                                continue;
                            }
                            _ => {
                                return Ok(Some(mqtt_msg));
                            }
                        }
                    }
                }

                // ignore pings
                Event::Outgoing(Outgoing::PingReq | Outgoing::PingResp)
                | Event::Incoming(Incoming::PingReq | Incoming::PingResp) => {
                    //return Ok(None);
                    continue;
                }
                // log everything else
                event => trace!("rumqttc: received event: {:?}", event),
            }

            // Ok(None)
        }
    }

    ///Parsed Json zu Message anhand von From und Type.
    fn parse_message(
        &mut self,
        _topic: &str,
        mqtt_payload: &[u8],
    ) -> Result<Option<Self::M>, Box<dyn core::error::Error>> {
        let message: serde_json::Value = serde_json::from_slice(mqtt_payload)?;
        let from: From = serde_json::from_value(message["from"].clone())?;
        let typ: Typ = serde_json::from_value(message["type"].clone())?;
        let payload = &message["payload"];
        let parsed = match (from, typ) {
            (From::Server, Typ::Planet) => {
                Some(Payload::Planet(serde_json::from_value(payload.clone())?))
            }

            (From::Server, Typ::PathSelect) => Some(Payload::PathSelectServer(
                serde_json::from_value(payload.clone())?,
            )),

            (From::Server, Typ::Path) => Some(Payload::PathServer(serde_json::from_value(
                payload.clone(),
            )?)),

            (From::Server, Typ::PathUnveiled) => {
                Some(Payload::PathUnveiled(serde_json::from_value::<
                    PathUnveiledPayload,
                >(payload.clone())?))
            }

            (From::Server, Typ::Target) => {
                Some(Payload::Target(serde_json::from_value(payload.clone())?))
            }
            (From::Server, Typ::Done) => {
                Some(Payload::Done(serde_json::from_value(payload.clone())?))
            }

            (From::Debug, Typ::Notice) => {
                Some(Payload::Notice(serde_json::from_value(payload.clone())?))
            }

            (From::Debug, Typ::Error) => {
                Some(Payload::Error(serde_json::from_value(payload.clone())?))
            }

            (From::Server, Typ::Syntax) => {
                Some(Payload::Syntax(serde_json::from_value(payload.clone())?))
            }
            _ => None,
        };

        Ok(Some(Message {
            from,
            typ,
            payload: parsed,
        }))
    }

    ///Subscribed dem aktuellem Topic, Entweder "explorer/004" oder "planet/PLANET_NAME/004"
    fn subscribe(&mut self, topic: &str) -> Result<(), Box<dyn core::error::Error>> {
        self.client
            .subscribe(topic, mqtt::QoS::ExactlyOnce)
            .map_err(Box::new)?;
        Ok(())
    }
    ///Unsubscribed dem aktuellem Topic, Entweder "explorer/004" oder "planet/PLANET_NAME/004"
    fn unsubscribe(&mut self, topic: &str) -> Result<(), Box<dyn core::error::Error>> {
        self.client.unsubscribe(topic).map_err(Box::new)?;
        Ok(())
    }

    ///Sendet Nachricht zum angegebenem Topic. Vorher muss dem Topic subscribed werden.
    fn publish(
        &mut self,
        topic: &str,
        payload: Vec<u8>,
    ) -> Result<(), Box<dyn core::error::Error>> {
        while let Ok(Ok(event)) = self.connection.recv_timeout(Duration::from_millis(500)) {
            match event {
                Event::Incoming(Incoming::Publish(_)) => println!("Found unhandled Message!"),
                Event::Outgoing(Outgoing::Subscribe(_)) => println!("Found unhandled Subscribe!"),
                Event::Outgoing(Outgoing::Publish(_)) => println!("Found unhandled Publish!"),
                _ => (),
            }
        }

        self.client
            .publish(topic, QoS::ExactlyOnce, false, payload)
            .map_err(EasyError::Client)?;
        while let Ok(Ok(event)) = self.connection.recv_timeout(Duration::from_millis(500)) {
            match event {
                Event::Incoming(Incoming::Publish(_)) => println!("Found unhandled Message!"),
                Event::Outgoing(Outgoing::Subscribe(_)) => println!("Found unhandled Subscribe!"),
                Event::Outgoing(Outgoing::Publish(_)) => break,
                _ => (),
            }
        }

        Ok(())
    }
}

impl Communication {
    /// Initializes communication module, connect to server, subscribe, etc.
    pub(crate) fn new(client: mqtt::Client, connection: mqtt::Connection) -> Self {
        Self { client, connection }
    }

    ///Erstellt test planet Message
    pub fn test_planet(planet_name: &str) -> Message {
        Message {
            from: From::Client,
            typ: Typ::TestPlanet,
            payload: Some(Payload::TestPlanetPayload(TestPlanetPayload {
                planet_name: planet_name.to_string(),
            })),
        }
    }

    ///Erstellt ready Message
    pub fn ready() -> Message {
        Message {
            from: From::Client,
            typ: Typ::Ready,
            payload: None,
        }
    }

    ///Erstellt path_select Message
    pub fn path_select(xs: i32, ys: i32, ds: i32) -> Message {
        Message {
            from: From::Client,
            typ: Typ::PathSelect,
            payload: Some(Payload::PathSelectClient(PathSelectClientPayload {
                start_x: xs,
                start_y: ys,
                start_direction: ds,
            })),
        }
    }

    ///Erstellt path_select Message
    #[allow(clippy::too_many_arguments)]
    pub fn explored_path(
        xs: i32,
        ys: i32,
        ds: i32,
        xe: i32,
        ye: i32,
        de: i32,
        path_status: PathStatus,
    ) -> Message {
        Message {
            from: From::Client,
            typ: Typ::Path,
            payload: Some(Payload::PathClient(PathClientPayload {
                start_x: xs,
                start_y: ys,
                start_direction: ds,
                end_x: xe,
                end_y: ye,
                end_direction: de,
                path_status,
            })),
        }
    }

    ///Erstellt target_reached Message
    pub fn target_reached() -> Message {
        Message {
            from: From::Client,
            typ: Typ::TargetReached,
            payload: Some(Payload::TargetReached(TargetReachedPayload {
                message: "Target reached yeheheha :)".to_string(),
            })),
        }
    }

    ///Erstellt exploration_completed Message
    pub fn exploration_completed() -> Message {
        Message {
            from: From::Client,
            typ: Typ::ExplorationCompleted,
            payload: Some(Payload::ExplorationCompleted(ExplorationCompletedPayload {
                message: "Exploration completed! yeheheha :)".to_string(),
            })),
        }
    }

    //-----------------------------------------------Extra Version, die die Nachricht auch sofort verschickt----------------//

    ///Erstellt test planet Message und verschickt sie
    pub fn send_test_planet(&mut self, planet_name: &str) {
        let test_planet_message = Communication::test_planet(planet_name);
        let test_planet_bytes = serde_json::to_vec(&test_planet_message).unwrap();
        self.subscribe("explorer/004").unwrap();
        self.publish("explorer/004", test_planet_bytes).unwrap();
    }

    ///Erstellt readyt Message und verschickt sie
    pub fn send_ready(&mut self) {
        let ready_message = Communication::ready();
        let ready_message_bytes = serde_json::to_vec(&ready_message).unwrap();
        self.subscribe("explorer/004").unwrap();
        self.publish("explorer/004", ready_message_bytes).unwrap();
    }

    ///Erstellt path_select Message und verschickt sie
    pub fn send_path_select(&mut self, xs: i32, ys: i32, ds: i32, topic: &str) {
        let path_selected_message = Communication::path_select(xs, ys, ds);
        let path_select_bytes = serde_json::to_vec(&path_selected_message).unwrap();
        self.subscribe(topic).unwrap();
        self.publish(topic, path_select_bytes).unwrap();
    }

    ///Erstellt explored_path Message und verschickt sie
    #[allow(unused, clippy::too_many_arguments)]
    pub fn send_explored_path(
        &mut self,
        xs: i32,
        ys: i32,
        ds: i32,
        xe: i32,
        ye: i32,
        de: i32,
        path_status: PathStatus,
        topic: &str,
    ) {
        let explored_path_message =
            Communication::explored_path(xs, ys, ds, xe, ye, de, path_status);
        let explored_path_bytes = serde_json::to_vec(&explored_path_message).unwrap();
        self.subscribe(topic).unwrap();
        self.publish(topic, explored_path_bytes).unwrap();
    }

    ///Erstellt target_reached Message und verschickt sie
    pub fn send_target_reached(&mut self) {
        let target_reached_message = Communication::target_reached();
        let target_reached_bytes = serde_json::to_vec(&target_reached_message).unwrap();
        self.subscribe("explorer/004").unwrap();
        self.publish("explorer/004", target_reached_bytes).unwrap();
    }

    ///Erstellt exploration_completed Message und verschickt sie
    pub fn send_exploration_completed(&mut self) {
        let exploration_completed_message = Communication::exploration_completed();
        let exploration_complete_bytes =
            serde_json::to_vec(&exploration_completed_message).unwrap();
        self.subscribe("explorer/004").unwrap();
        self.publish("explorer/004", exploration_complete_bytes)
            .unwrap();
    }

    ///Muss nach send_explored_path aufgerufen werden. Empfängt alle Server und Debug Nachrichten
    pub fn multiple_messages(&mut self) -> Vec<Message> {
        let mut messages = vec![];
        while let Ok(Some(msg)) = self.receive_message(Duration::from_secs(3)) {
            let message = self
                .parse_message("juckt nicht", &msg.payload)
                .unwrap()
                .unwrap();
            messages.push(message);
        }
        messages
    }
    //Message handling-------------------------------Message handling--------------------------------------------Message handling

    ///Prüft Typ der Nachricht und ruft passenden handler auf
    pub fn handle_message(
        &mut self,
        message: Message,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        match message {
            Message {
                from: From::Server,
                typ: Typ::Planet,
                payload: Some(Payload::Planet(payload)),
            } => self.handle_planet(payload),

            Message {
                from: From::Server,
                typ: Typ::PathSelect,
                payload: Some(Payload::PathSelectServer(payload)),
            } => self.handle_path_select_server(payload),

            Message {
                from: From::Server,
                typ: Typ::Path,
                payload: Some(Payload::PathServer(payload)),
            } => self.handle_path_server(payload),

            Message {
                from: From::Server,
                typ: Typ::PathUnveiled,
                payload: Some(Payload::PathUnveiled(payload)),
            } => self.handle_path_unveiled(payload),

            Message {
                from: From::Server,
                typ: Typ::Target,
                payload: Some(Payload::Target(payload)),
            } => self.handle_target(payload),

            Message {
                from: From::Server,
                typ: Typ::Done,
                payload: Some(Payload::Done(payload)),
            } => self.handle_done(payload),

            Message {
                from: From::Debug,
                typ: Typ::Notice,
                payload: Some(Payload::Notice(payload)),
            } => self.handle_notice(payload),

            Message {
                from: From::Debug,
                typ: Typ::Syntax,
                payload: Some(Payload::Syntax(payload)),
            } => self.handle_syntax(payload),

            Message {
                from: From::Debug,
                typ: Typ::Error,
                payload: Some(Payload::Error(payload)),
            } => self.handle_error(payload),

            _ => Ok(None),
        }
    }

    fn handle_planet(
        &mut self,
        payload: PlanetPayload,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        println!("{}", payload.planet_name);
        println!("{}", payload.start_x);
        println!("{}", payload.start_y);
        println!("{}", payload.start_orientation);
        Ok(None)
    }

    fn handle_path_select_server(
        &mut self,
        payload: PathSelectServerPayload,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        //
        println!("Start Direction: {}", payload.start_direction);
        Ok(None)
    }

    fn handle_path_server(
        &mut self,
        payload: PathServerPayload,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        //Wert zurückgeben
        println!("SX: {}", payload.start_x);
        println!("SY: {}", payload.start_y);
        println!("SD: {}", payload.start_direction);
        println!("EX: {}", payload.end_x);
        println!("EY: {}", payload.end_y);
        println!("ED: {}", payload.end_direction);
        println!("PT: {:?}", payload.path_status);
        println!("PW: {:?}", payload.path_weight);
        Ok(None)
    }

    fn handle_path_unveiled(
        &mut self,
        payload: PathUnveiledPayload,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        //Neuen Pfad erstellen mit den Werten
        println!("Payload: {:?}", payload);
        Ok(None)
    }

    fn handle_target(
        &mut self,
        _payload: TargetPayload,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        //Neue Node mit target x und target y
        //Diese an shortest path übergeben
        println!("target");
        Ok(None)
    }

    fn handle_done(
        &mut self,
        _payload: DonePayload,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        //
        println!("done");
        Ok(None)
    }

    fn handle_notice(
        &mut self,
        _payload: NoticePayload,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        println!("notice");
        Ok(None)
    }

    fn handle_syntax(
        &mut self,
        _payload: SyntaxPayload,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        println!("syntax");
        Ok(None)
    }

    fn handle_error(
        &mut self,
        _payload: ErrorPayload,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        println!("error");
        Ok(None)
    }
}
