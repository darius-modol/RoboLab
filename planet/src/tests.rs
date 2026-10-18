//! This module (and possible submodules) should contain
//! your tests for your BotPlanet implementation.
//!
//! Our tests use the builtin `#[test]` attribute macro.
//! You MUST NOT change the functions signatures of our tests!
//!
//! You may use `rstest` **for your own tests** and make use of fixtures.
//! See the [rstest documentation](https://docs.rs/rstest) for more info.
//! However, DO NOT change the attribute macro on the provided test functions.

use super::create_planet;
use robolab_bot_base::planet::{BotPlanet, Direction, Node, OutgoingPaths};
use std::collections::HashMap;

/// Instantiates the planet data structure and fills it with paths
///
/// ```
/// +--+
/// |  |
/// +-0,3------+
///    |       |
///   0,2-----2,2 (target)
///    |      /
/// +-0,1    /
/// |  |    /
/// +-0,0-1,0
///    |
/// (start)
/// ```
///
fn setup_test_planet() -> Box<dyn BotPlanet> {
    // Initialize your data structure here
    let mut my_planet = create_planet();
    my_planet.add_path(((0, 0), Direction::East), ((10, 0), Direction::West), 3);
    my_planet.add_path(((0, 0), Direction::West), ((0, 0), Direction::North), 1);
    my_planet.add_path(((0, 0), Direction::South), ((-3, 15), Direction::South), 6);
    my_planet.add_path(((10, 0), Direction::South), ((-3, 15), Direction::West), 3);
    my_planet.add_path(((-3, 15), Direction::East), ((20, 0), Direction::South), -1);
    my_planet.add_path(((10, 0), Direction::East), ((20, 0), Direction::West), 5);
    my_planet.add_path(((10, 0), Direction::North), ((10, 5), Direction::South), 20);
    my_planet.add_path(((10, 5), Direction::West), ((10, 5), Direction::West), 20);
    my_planet.add_path(((10, 5), Direction::East), ((20, 0), Direction::North), 1);
    my_planet.add_path(((10, 5), Direction::North), ((20, 10), Direction::West), 3);
    my_planet.add_path(((20, 10), Direction::East), ((20, 0), Direction::East), 6);
    my_planet.add_path(
        ((20, 10), Direction::North),
        ((20, 15), Direction::South),
        1,
    );
    // ...
    my_planet
}

/// This test should check that the shortest-path algorithm does not get stuck in a loop between two points while
/// searching for a target not reachable nearby
///
/// Result: Target is not reachable
#[test]
#[ignore = "Example test, should not count in final test results"]
fn example_test_target_not_reachable_with_loop() {
    let planet = setup_test_planet();
    assert_eq!(planet.shortest_path((0, 0), (1, 2)), None)
}

/// This test should check that the dictionary returned by "planet.get_paths()" matches the expected structure
#[test]
fn test_integrity() {
    //unimplemented!()
    let mut first_planet = create_planet();
    let mut inner_hm1: OutgoingPaths = HashMap::new();
    let mut inner_hm2: OutgoingPaths = HashMap::new();
    let mut inner_hm3: OutgoingPaths = HashMap::new();

    let mut outer_hm: HashMap<Node, OutgoingPaths> = HashMap::new();
    inner_hm1.insert(Direction::North, ((1, 1), Direction::West, 1));
    inner_hm1.insert(Direction::East, ((1, 0), Direction::West, 2));
    outer_hm.insert((0, 0), inner_hm1);
    inner_hm2.insert(Direction::West, ((0, 0), Direction::North, 1));
    inner_hm2.insert(Direction::South, ((1, 0), Direction::North, 4));
    inner_hm2.insert(Direction::East, ((1, 0), Direction::East, 3));
    outer_hm.insert((1, 1), inner_hm2);
    inner_hm3.insert(Direction::West, ((0, 0), Direction::East, 2));
    inner_hm3.insert(Direction::North, ((1, 1), Direction::South, 4));
    inner_hm3.insert(Direction::East, ((1, 1), Direction::East, 3));
    outer_hm.insert((1, 0), inner_hm3);

    first_planet.add_path(((0, 0), Direction::North), ((1, 1), Direction::West), 1);
    first_planet.add_path(((0, 0), Direction::East), ((1, 0), Direction::West), 2);
    first_planet.add_path(((1, 1), Direction::South), ((1, 0), Direction::North), 4);
    first_planet.add_path(((1, 0), Direction::East), ((1, 1), Direction::East), 3);

    assert_eq!(first_planet.get_paths(), outer_hm)
}

/// This test should check that an empty planet really is empty
#[test]
fn test_empty_planet() {
    //unimplemented!()
    let second_planet = create_planet();
    assert_eq!(second_planet.get_paths().len(), 0)
}

/// This test should check that the shortest-path algorithm implemented works.
///
/// Requirement: Minimum distance is three nodes (two paths in list returned)
#[test]
fn test_target() {
    let third_planet = setup_test_planet();
    //unimplemented!()
    assert_eq!(
        third_planet.shortest_path((0, 0), (20, 0)),
        Some(vec!(((0, 0), Direction::East), ((10, 0), Direction::East)))
    )
}

/// This test should check that a target outside the map or at an unexplored node is not reachable
#[test]
fn test_target_not_reachable() {
    let fourth_planet = setup_test_planet();
    assert_eq!(fourth_planet.shortest_path((0, 0), (10, 10)), None)
    //unimplemented!()
}

/// This test should check that the shortest-path algorithm implemented returns the shortest path even if there
/// are multiple shortest paths with the same length.
///
/// Requirement: Minimum of two paths with same cost exists, only one is returned by the logic implemented
#[test]
fn test_same_length() {
    //unimplemented!()
    let fifth_planet = setup_test_planet();
    assert_eq!(
        fifth_planet.shortest_path((-3, 15), (0, 0)),
        Some(vec!(((-3, 15), Direction::South)))
    )
}

/// This test should check that the shortest-path algorithm does not get stuck in a loop between two points while
/// searching for a target nearby
///
/// Result: Target is reachable
#[test]
fn test_target_with_loop() {
    //unimplemented!()
    let mut sixth_planet = create_planet();
    sixth_planet.add_path(((0, 0), Direction::North), ((0, 1), Direction::South), 1);
    sixth_planet.add_path(
        ((0, 1), Direction::East),
        ((1, 1), Direction::West),
        1000000,
    );
    assert_eq!(
        Some(vec!(((0, 0), Direction::North), ((0, 1), Direction::East))),
        sixth_planet.shortest_path((0, 0), (1, 1))
    )
}

/// This test should check that the shortest-path algorithm does not get stuck in a loop between two points while
/// searching for a target not reachable nearby
///
/// Result: Target is not reachable
#[test]
fn test_target_not_reachable_with_loop() {
    //unimplemented!()
    let mut seventh_planet = create_planet();
    seventh_planet.add_path(((0, 1), Direction::South), ((0, 0), Direction::North), 3);
    assert_eq!(seventh_planet.shortest_path((0, 1), (1, 1)), None)
}
