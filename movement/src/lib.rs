use crate::hardware::ui::UI;
use robolab_bot_base::movement::BotMovement;
use std::rc::Rc;

pub mod hardware;
mod movement_impl;
pub mod odometry;
mod pid_controller;

pub fn create_movement(speed: i32, ui: Rc<UI>) -> Box<dyn BotMovement> {
    let movement = movement_impl::Movement::new(speed, ui);
    Box::new(movement)
}
