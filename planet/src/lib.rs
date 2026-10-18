use robolab_bot_base::planet::BotPlanet;

#[cfg(test)]
mod tests;

pub mod planet_impl;

// DO NOT EDIT function signature, only edit the function body
pub fn create_planet() -> Box<dyn BotPlanet> {
    let planet = Planet::new();
    Box::new(planet)
}

pub use crate::planet_impl::NodeInQueue;
pub use crate::planet_impl::Planet;

pub fn create_test() -> Box<Planet> {
    let planet = Planet::new();
    Box::new(planet)
}

pub fn just_planet() -> Planet {
    Planet::new()
}
