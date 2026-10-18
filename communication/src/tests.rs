//! This module (and possible submodules) may contain
//! your tests for parsing MQTT Messages implementation.
//!
//! IMPORTANT: THOSE TESTS ARE NOT REQUIRED FOR THE EXAM AND USED ONLY FOR DEVELOPMENT
//!
//! You may use `rstest` **for your own tests** and make use of fixtures.
//! See the [rstest documentation](https://docs.rs/rstest) for more info.
use super::Message;
use crate::communication_impl::*;
use robolab_bot_base::communication::BotCommunication;
use rumqttc as mqtt;
use serde_json::json;
use std::time::Duration;


fn create_connection() -> Communication {
    let mut opts = mqtt::MqttOptions::new("004", "mothership.inf.tu-dresden.de", 8883);
    opts.set_transport(mqtt::Transport::tls_with_default_config())
        .set_clean_session(true)
        .set_keep_alive(Duration::from_secs(60))
        .set_credentials("004", "<redacted>");

    let (client, connection) = mqtt::Client::new(opts, 1 << 12);
    Communication::new(client, connection)
}

//testet ob test planet verschickt und passende Antwort empfangen werden kann
#[test]
fn test_publish_test_planet() {
    let mut comm = create_connection();
    comm.send_test_planet("Wirth");
    let msg = comm
        .receive_message(Duration::from_secs(20))
        .unwrap()
        .expect("Failed to receive message");
    let message = comm
        .parse_message("explorer/004", &msg.payload)
        .unwrap()
        .unwrap();
    println!("{:?}", message)
}

//testet ob ready verschickt und passende Antwort empfangen werden kann
#[test]
fn test_publish_ready() {
    let mut comm = create_connection();
    comm.send_ready();
    let msg = comm
        .receive_message(Duration::from_secs(20))
        .unwrap()
        .expect("Failed to receive message");
    let message = comm
        .parse_message("explorer/004", &msg.payload)
        .unwrap()
        .unwrap();
    println!("{:?}", message);
}

//testet ob path select verschickt und passende Antwort empfangen werden kann
#[test]
fn test_publish_path_select() {
    let mut comm = create_connection();
    comm.send_path_select(13, 2, 0, "planet/Anin/004");
    let msg = comm
        .receive_message(Duration::from_secs(20))
        .unwrap()
        .expect("Failed to receive message");
    let message = comm
        .parse_message("planet/Anin/004", &msg.payload)
        .unwrap()
        .unwrap();
    println!("{:?}", message);
}

//testet ob path verschickt und passende Antwort empfangen werden kann
#[test]
fn test_publish_path() {
    let mut comm = create_connection();
    comm.send_test_planet("Anin");
    comm.send_ready();
    comm.send_explored_path(15, 2, 270, 13, 2, 0, PathStatus::Free, "planet/Anin/004");
    if let Some(msg) = comm.receive_message(Duration::from_secs(20)).unwrap() {
        let message = comm
            .parse_message("planet/Anin/004", &msg.payload)
            .unwrap()
            .unwrap();
        println!("{:?}", message);
    }
}


//testet ob target reached verschickt und passende Antwort empfangen werden kann
#[test]
fn test_publish_target_reached() {
    let mut comm = create_connection();
    comm.send_target_reached();
    if let Some(msg) = comm.receive_message(Duration::from_secs(20)).unwrap() {
        let message = comm
            .parse_message("explorer/004", &msg.payload)
            .unwrap()
            .unwrap();
        println!("{:?}", message);
    }
}

//testet ob exploration complete verschickt und passende Antwort empfangen werden kann
#[test]
fn test_publish_exploration_completed() {
    let mut comm = create_connection();
    comm.send_exploration_completed();
    if let Some(msg) = comm.receive_message(Duration::from_secs(20)).unwrap() {
        let message = comm
            .parse_message("explorer/004", &msg.payload)
            .unwrap()
            .unwrap();
        println!("{:?}", message);
    }
}

#[test]
fn serialize_ready_message() {
    let test = Message {
        from: From::Client,
        typ: Typ::Ready,
        payload: None,
    };
    let value = serde_json::to_value(&test).unwrap();
    assert_eq!(
        value,
        json!({
         "from": "client",
         "type": "ready"
        })
    );
}

#[test]
fn serialize_path_unveiled_message() {
    let test = Message {
        from: From::Server,
        typ: Typ::PathUnveiled,
        payload: Some(Payload::PathUnveiled(PathUnveiledPayload {
            start_x: 6,
            start_y: 7,
            start_direction: 0,
            end_x: 6,
            end_y: 9,
            end_direction: 67,
            path_status: PathStatus::Free,
            path_weight: 5,
        })),
    };
    let json = serde_json::to_value(&test).unwrap();
    assert_eq!(
        json,
        json!({
          "from": "server",
          "type": "pathUnveiled",
          "payload": {
            "startX": 6,
            "startY": 7,
            "startDirection": 0,
            "endX": 6,
            "endY": 9,
            "endDirection": 67,
            "pathStatus": "free",
            "pathWeight": 5
          }
        })
    );
}

#[test]
fn serialize_path_message() {
    let test = Message {
        from: From::Server,
        typ: Typ::Path,
        payload: Some(Payload::PathServer(PathServerPayload {
            start_x: 3,
            start_y: 4,
            start_direction: 5,
            end_x: 6,
            end_y: 7,
            end_direction: 8,
            path_status: PathStatus::Free,
            path_weight: 10,
        })),
    };
    let json = serde_json::to_value(&test).unwrap();
    assert_eq!(
        json,
        json!({
          "from": "server",
          "type": "path",
          "payload": {
            "startX": 3,
            "startY": 4,
            "startDirection": 5,
            "endX": 6,
            "endY": 7,
            "endDirection": 8,
            "pathStatus": "free",
            "pathWeight": 10
          }
        })
    )
}

#[test]
fn debug_parse_ready_message() {
    let mut comm = create_connection();
    let byte_string = br#"{
    "from": "client",
    "type": "ready"
  }"#;
    let debug_message = comm
        .parse_message("juckt nicht", byte_string)
        .unwrap()
        .expect("Fehler (bitte nicht)");
    println!("{:?}", debug_message);
    assert!(matches!(debug_message.from, From::Client));
    assert!(matches!(debug_message.typ, Typ::Ready));
    assert!(debug_message.payload.is_none());
}
//
#[test]
fn debug_parse_path_unveiled_message() {
    let byte_string = br#"{
   "from": "server",
   "type": "pathUnveiled",
   "payload": {
     "startX": 3,
     "startY": 4,
     "startDirection": 5,
     "endX": 6,
     "endY": 7,
     "endDirection": 8,
     "pathStatus": "free",
     "pathWeight": 10
     }
   }"#;
    let mut comm = create_connection();
    let debug_message = comm.parse_message("lol", byte_string).unwrap().unwrap();
    println!("Parsed Message: {:?}", debug_message);
}

#[test]
fn debug_parse_target_message() {
    let byte_string = br#"{
  "from": "server",
  "type": "target",
  "payload": {
     "targetX": 3,
     "targetY": 4
 }
   }"#;
    let mut comm = create_connection();
    let debug_message = comm.parse_message("lol", byte_string).unwrap().unwrap();
    println!("Parsed Message: {:?}", debug_message)
}

#[test]
fn debug_done_message() {
    let byte_string = br#"{
  "from": "server",
   "type": "done",
   "payload": {
     "message": "yeeeeee"
 }
   }"#;
    let mut comm = create_connection();
    let debug_message = comm.parse_message("lol", byte_string).unwrap().unwrap();
    println!("payload: {:?}", debug_message);
}

//testet ob ready Antwort richtig geparsed werden kann
#[test]
fn test_message_handler_ready() {
    let mut comm = create_connection();
    comm.send_ready();
    let msg = comm
        .receive_message(Duration::from_secs(3))
        .unwrap()
        .expect("Failed to receive message");

    let message = comm
        .parse_message("explorer/004", &msg.payload)
        .unwrap()
        .unwrap();
   comm.handle_message(message).unwrap();
}

//testet ob path_select Antwort richtig geparsed werden kann
#[test]
fn test_message_handler_path_select() {
    let mut comm = create_connection();
    comm.send_path_select(13, 2, 0, "planet/Anin/004");
    let msg = comm
        .receive_message(Duration::from_secs(20))
        .unwrap()
        .expect("Failed to receive message");

    let message = comm
        .parse_message("planet/Anin/004", &msg.payload)
        .unwrap()
        .unwrap();
    comm.handle_message(message).unwrap();
}

//testet ob path Antwort richtig geparsed werden kann
#[test]
fn test_message_handler_path() {
    let mut comm = create_connection();
    comm.send_test_planet("Anin");
    comm.send_ready();
    comm.send_explored_path(15, 2, 270, 13, 2, 0, PathStatus::Free, "planet/Anin/004");
    let msg = comm
        .receive_message(Duration::from_secs(20))
        .unwrap()
        .expect("Failed to receive message");

    let message = comm
        .parse_message("planet/Anin/004", &msg.payload)
        .unwrap()
        .unwrap();
    comm.handle_message(message).unwrap();
}

//testet ob ob mehere Nachrichten ankommen und richtig geparsed werden
#[test]
fn test_multiple_messages() {
    let mut comm = create_connection();
    comm.send_ready();
    comm.send_explored_path(15, 2, 270, 13, 2, 0, PathStatus::Free, "planet/Anin/004");
    let recived_messages = comm.multiple_messages();
    println!("recieved: {:?}", recived_messages);
    let mut messages = Vec::new();
    for m in recived_messages {
        messages.push(comm.handle_message(m).unwrap())
    }
}
