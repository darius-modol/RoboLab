extern crate communication;
extern crate movement;
extern crate planet;
mod communication_handler;

use crate::communication_handler::CommunicationHandler;
use communication::{Message, PathStatus};
use ev3dev_lang_rust::sound::play;
use movement::hardware::ui::UI;
use movement::odometry::Odometry;
use planet::Planet;
use robolab_bot_base::communication::BotCommunication;
use robolab_bot_base::movement::{BotMovement, OdometryData};
use robolab_bot_base::planet::{Direction, Node, ShortestPath};
use robolab_bot_base::setup;
use std::rc::Rc;
use std::time::Duration;

static GROUP_ID: &str = "004";
static EXPLORER_TOPIC: &str = "explorer/004";
static TIMEOUT: Duration = Duration::from_secs(6);

pub fn main() {
    setup::init_logging();
    setup::write_pidfile();

    let mut c = communication::create_communication();
    let ui = Rc::new(UI::new());
    let mut p = Planet::new();
    let speed = 50;
    let mut m = movement::create_movement(speed, ui.clone());

    let planet_name = "EXAM";

    let (planet_name, mut x, mut y, mut direction) =
        initialize_exploration(planet_name, &mut c, &mut m);

    direction = int_to_direction((direction as i32 + 180).rem_euclid(360)).unwrap();

    let mut target: Option<Node> = None;

    // If target is some this is `path to target`
    // If target is none this is `smart select`
    let mut path_to_follow: Option<ShortestPath> = None;

    loop {
        if !p.is_scanned((x as i64, y as i64)) {
            // scan node for paths
            println!("Scanning node at {}, {}", x, y);
            let paths = m.scan_nodes();
            println!("Scanned node: {:?}", paths);

            // Insert path into planet structure except the one we are coming from
            paths
                .iter()
                .flatten()
                .filter(|d| **d as i32 != direction as i32)
                .fold((), |_, dir| {
                    println!("Adding path to direction: {:?}", dir);
                    let path = ((x as i64, y as i64), *dir);
                    p.add_path(path, path, 0);
                });

            p.scanned((x as i64, y as i64));
        }

        // select path - either path to target or smart exploration
        // exists when finished
        if let Some((dir, path)) =
            smart_select_direction(&mut c, &mut p, &planet_name, (x, y), target, path_to_follow)
        {
            direction = dir;
            path_to_follow = path;
            println!("Trying to drive {:?}", direction);
        } else {
            // Shouldn't ever be reached `smart_select_direction` is either Some or exits
            println!("Exploration completed!");
            return;
        }

        // send path select and follow `suggested` path if available
        if let Some(forced_direction) =
            CommunicationHandler::send_select_path(&mut c, x, y, direction, &planet_name)
        {
            if let Ok(mut sound) = play("./sounds/server-path-select/samsung.wav") {
                let _ = sound.wait();
            }
            direction = forced_direction;
            path_to_follow = None;
            println!("path_select - forced to drive {:?}", direction);
        } else {
            println!("no path_select continuing {:?}", direction);
        }

        // follow line to next node
        let odometry = m.follow_line(direction);
        let (ex, ey, ed, weight) = path_explored(&mut c, &planet_name, x, y, direction, odometry);

        println!(
            "explored path {}, {}, {:?} - {}, {}, {:?} - weight: {}",
            x, y, direction, ex, ey, ed, weight
        );
        // Add explored path to planet structure
        p.add_path(
            ((x as i64, y as i64), direction),
            ((ex as i64, ey as i64), int_to_direction(ed).unwrap()),
            weight as i64,
        );

        // Set `x` and `y` and `direction` to corrected values from the server
        x = ex;
        y = ey;
        direction = int_to_direction(ed).unwrap();
        println!("robot is now at {}, {}", x, y);

        // listen for unveiled paths and targets
        let (paths, target_opt) =
            CommunicationHandler::listen_for_paths(&mut c, &planet_name, TIMEOUT);
        if !paths.is_empty() {
            path_to_follow = None;
            if let Ok(mut sound) = play("./sounds/path-unveiled/deutsche-bahn.wav") {
                let _ = sound.wait();
            }
        }
        paths.iter().fold((), |_, payload| {
            println!("Server unveiled path {:?}", payload);
            // Adding paths to planet structure
            p.add_path(
                (
                    (payload.start_x as i64, payload.start_y as i64),
                    int_to_direction(payload.start_direction).unwrap(),
                ),
                (
                    (payload.end_x as i64, payload.end_y as i64),
                    int_to_direction(payload.end_direction).unwrap(),
                ),
                payload.path_weight as i64,
            );
        });
        // Handle targets sent by server
        if let Some(t) = target_opt {
            println!("Server sent target {:?}", t);
            if let Ok(mut sound) = play("./sounds/odometry-wrong/amogus.wav") {
                let _ = sound.wait();
            }
            target = Some((t.target_x as i64, t.target_y as i64));
            path_to_follow = None;
        }
    }
}

fn smart_select_direction(
    c: &mut Box<dyn BotCommunication<M = Message>>,
    p: &mut Planet,
    planet_name: &str,
    pos: (i32, i32),
    target: Option<Node>,
    path_to_target: Option<ShortestPath>,
) -> Option<(Direction, Option<ShortestPath>)> {
    let mut path_to_target = path_to_target;
    let (x, y) = pos;

    // Find way to target
    if let Some(target) = target {
        println!("Target available: {:?}", target);
        // If no path to target, try to calculate it
        if path_to_target.is_none() {
            if let Some(path) = p.shortest_path((x as i64, y as i64), target) {
                println!("Calculated path to target");
                // Set path to target
                path_to_target = Some(path);
            } else {
                println!("No path to target available, trying smart explore");
                // If we can't calculate it we use smart explore
                // But set path_to_target None and try calculating it again at the next node
                return if let Some(mut path_to_unexplored) = p.smart_select((x as i64, y as i64))
                    && !path_to_unexplored.is_empty()
                {
                    Some((path_to_unexplored.remove(0).1, None))
                } else {
                    println!("Also no smart explore available");
                    println!("Exploration complete!");
                    CommunicationHandler::send_exploration_completed(c, planet_name);
                    None
                };
            }
        }

        // try to follow path to target
        let mut path = path_to_target.unwrap();
        if path.is_empty() {
            println!("target reached!");
            CommunicationHandler::send_target_reached(c, planet_name);
            None
        } else {
            let direction = path.remove(0).1;
            // Return path to target as selected path
            println!(
                "Following path to target - direction: {:?}, path: {:?}",
                direction, path
            );
            Some((direction, Some(path)))
        }
    } else {
        // No target available - using smart explore
        if let Some(mut path_to_unexplored) = p.smart_select((x as i64, y as i64))
            && !path_to_unexplored.is_empty()
        {
            // Return smart explore as selected path
            Some((path_to_unexplored.remove(0).1, None))
        } else {
            // No paths to explore - seems we are finished
            println!("Exploration completed");
            CommunicationHandler::send_exploration_completed(c, planet_name);
            None
        }
    }
}

fn path_explored(
    c: &mut Box<dyn BotCommunication<M = Message>>,
    planet_name: &str,
    x: i32,
    y: i32,
    direction: Direction,
    odometry: Option<Vec<OdometryData>>,
) -> (i32, i32, i32, i32) {
    if let Some(mut odo) = odometry {
        // reached next node
        let (end_dir, _) = Odometry::odo_to_xy(odo.pop().unwrap());
        let (dx, dy) = Odometry::odo_to_xy(odo.pop().unwrap());

        println!(
            "Drove {:?} - ({}, {}) - coming from {}",
            direction, dx, dy, end_dir
        );

        let (ex, ey, ed, weight) = CommunicationHandler::send_explored_path(
            c,
            planet_name,
            x,
            y,
            direction as i32,
            x + dx,
            y + dy,
            end_dir,
            PathStatus::Free,
        )
        .expect("Failed to send explored path!");

        if ex != (x + dx) || ey != (y + dy) || ed != end_dir {
            if let Ok(mut sound) = play("./sounds/path-blocked/metal-pipe.wav") {
                let _ = sound.wait();
            }
        } else {
            if let Ok(mut sound) = play("./sounds/server-path-select/villager.wav") {
                let _ = sound.wait();
            }
        }

        (ex, ey, ed, weight)
    } else {
        // blocked path
        let _ = CommunicationHandler::send_explored_path(
            c,
            planet_name,
            x,
            y,
            direction as i32,
            x,
            y,
            direction as i32,
            PathStatus::Blocked,
        );
        (x, y, direction as i32, -1)
    }
}

fn initialize_exploration(
    _planet_name: &str,
    c: &mut Box<dyn BotCommunication<M = Message>>,
    m: &mut Box<dyn BotMovement>,
) -> (String, i32, i32, Direction) {
    println!("Driving to first node");
    let _ = m.follow_line(Direction::North); // Drive to first node

    c.subscribe(EXPLORER_TOPIC)
        .expect("Failed to subscribe to explorer topic");

    //TODO don't forget to remove this before exam
    //CommunicationHandler::send_set_planet(c, planet_name);
    println!("sending ready");

    let (planet_name, x, y, initial_direction) = CommunicationHandler::send_ready(c)
        .unwrap_or_else(|| CommunicationHandler::send_ready(c).expect("Couldn't send ready"));
    let _ = c.unsubscribe(EXPLORER_TOPIC);

    if let Ok(mut sound) = play("./sounds/target-reached/he-he-he-ha.wav") {
        let _ = sound.wait();
    }
    println!("Planet: {planet_name}, x: {x}, y:{y},initial_direction: {initial_direction}");
    let direction = int_to_direction(initial_direction).unwrap();
    let _ = m.follow_line(direction); // Hacky way to set reference angle -> does not move the robot
    if let Err(_e) = c.unsubscribe(EXPLORER_TOPIC) {
        eprintln!("Failed to unsubscribe from {}", EXPLORER_TOPIC)
    }

    (planet_name, x, y, direction)
}
fn int_to_direction(angle: i32) -> Option<Direction> {
    match angle {
        0 => Some(Direction::North),
        90 => Some(Direction::East),
        180 => Some(Direction::South),
        270 => Some(Direction::West),
        _ => None,
    }
}
