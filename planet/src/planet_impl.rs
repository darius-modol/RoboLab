use std::collections::{HashMap, HashSet};

use robolab_bot_base::RlPlanet;
use robolab_bot_base::planet::{BotPlanet, Direction, Node, OutgoingPaths, ShortestPath, Weight};

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Contains the representation of the map and provides certain functions to manipulate or extend
/// it according to the specifications
#[derive(Debug, Clone, RlPlanet)]
pub struct Planet {
    pub paths: HashMap<Node, OutgoingPaths>,
    pub scanned: HashSet<Node>,
}

// Struct fuer Elemente der Warteschlange des Dijkstra Algorithmus (Kuerzeste zurzeit bekannte Weg zu einer Node)
#[derive(Debug, Copy, Clone)]
pub struct NodeInQueue {
    weight: Weight,
    node: Node,
}

// Implementieren einer Ordnung da die Elemente (NodeInQueue) in einem BinaryHeap stehen welcher als die Warteschlange des Dijkstra handelt
// Jedes Element besteht aus einer Node und dem Gewicht des zurzeit mit minimalen Gewicht bekannten Weges vom Start zu dieser Node
// -1 als Gewicht fuer einen gesperrten Pfad und i64::MAX fuer unendlich
// Beachte!: damit es ein MinHeap wird, wird das Element welches den kleinsten positiven Wert an weight hat zum Groessten
// damit es ganz oben steht (Greater fuer: Element soll weiter vorne stehen also eher abgearbeitet werden)

impl Ord for NodeInQueue {
    fn cmp(&self, other: &Self) -> Ordering {
        let size1 = self.weight;
        let size2 = other.weight;

        if size1 < 0 && size2 > 0 {
            Ordering::Less
        } else if size1 > 0 && size2 < 0 {
            Ordering::Greater
        } else if size1 > 0 && size2 > 0 && size1 > size2 {
            Ordering::Less
        } else if size1 > 0 && size2 > 0 && size1 < size2 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for NodeInQueue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NodeInQueue {
    fn eq(&self, other: &Self) -> bool {
        (self.weight, &self.weight) == (other.weight, &other.weight)
    }
}

impl Eq for NodeInQueue {}

impl BotPlanet for Planet {
    fn add_path(&mut self, start: (Node, Direction), target: (Node, Direction), weight: Weight) {
        self.add_path(start, target, weight);
    }

    fn get_paths(&self) -> HashMap<Node, OutgoingPaths> {
        // self.paths enstpricht der Ausgabe der method get_paths
        self.get_paths()
    }
    fn shortest_path(&self, start: Node, target: Node) -> Option<ShortestPath> {
        self.shortest_path(start, target)
    }
}

impl Default for Planet {
    fn default() -> Self {
        Self::new()
    }
}

impl Planet {
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
            scanned: HashSet::new(),
        }
    }
    //Methode die im Feld scanned alle Nodes speichert, zu denen der Roboter auch wirklich gefahren ist
    //Falls bspw ein add_path() vom Server kommt, der Roboter aber noch nicht bei diesen Knoten war, muss er diesen trotzdem noch besuchen
    pub fn scanned(&mut self, node: Node) {
        println!("Scanned: {:?}", node);
        self.scanned.insert(node);
    }

    //damit Roboter Nodes nicht doppelt scannen muss
    pub fn is_scanned(&self, node: Node) -> bool {
        self.scanned.contains(&node)
    }

    pub fn get_paths(&self) -> HashMap<Node, OutgoingPaths> {
        // self.paths enstpricht der Ausgabe der method get_paths
        self.paths.clone()
    }

    pub fn add_path(
        &mut self,
        start: (Node, Direction),
        target: (Node, Direction),
        weight: Weight,
    ) {
        // destructen von start und target
        let (start_node, start_direction) = start;
        let (target_node, target_direction) = target;
        println!(
            "Pfad hinzufügen von: {:?} {:?} nach {:?} {:?}, weight: {:?}",
            start_node, start_direction, target_node, target_direction, weight
        );
        // special case: Gewicht 0 (kann normalerweise nicht passieren) Bedeutung: von diesem Knoten gibt es nicht besuchte Directions
        if weight == 0 {
            self.add_unexplored(start_node, start_direction);
        // normal case: einen Weg von einem zu einem anderen Knoten hinzufuegen
        } else {
            self.add_normal_path(
                start_node,
                start_direction,
                target_node,
                target_direction,
                weight,
            );
        }
    }

    pub fn shortest_path(&self, start: Node, target: Node) -> Option<ShortestPath> {
        println!("Shortest path von: {:?} zu {:?}", start, target);
        //Vektor erstellen, der alle existierenden Nodes enthaelt

        let mut visited_nodes: Vec<Node> = Vec::new();

        // HashMap die jeder Node ihren vorherigen Node zuordnet des aktuell am kuerzesten bekanntesten path und die Direction
        // vom Vorgaenger zu dieser Node
        let mut previous_node: HashMap<Node, (Node, Direction)> = HashMap::new();
        // HashMap die jeder Node das weight des aktuell kuerzesten bekannten Pfades zuordnet
        let mut total_weight: HashMap<Node, Weight> = HashMap::new();
        // Binary Heap aus allen noch nicht besuchten Nodes (PriorityQueue)
        let mut unvisited_nodes: BinaryHeap<NodeInQueue> = BinaryHeap::new();

        //Startnode bekommt die Entfernung 0, alle anderen Nodes die Entfernung unendlich (Maximum value)
        self.initialize_dijkstra(start, &mut total_weight, &mut unvisited_nodes);
        //Queue die immer das Node mit dem kleinsten weight als erstes waehlt und bearbeitet,
        // node_node ist die Node, dessen Nachbarn jetzt untersucht wird
        while let Some(NodeInQueue {
            weight: node_weight,
            node: node_node,
        }) = unvisited_nodes.pop()
        {
            //checken ob Node schonmal bearbeitet wurde, falls ja -> ueberspringen
            if visited_nodes.contains(&node_node) {
                continue;
            }
            // fals noch nicht bearbeitet, jetzt in schon besuchte Nodes hinzufuegen
            visited_nodes.push(node_node);

            // falls die zu untersuchende Node in den target_nodes ist => Algorithmus hat Ziel gefunden
            if node_node == target {
                return self.found_target_sp(start, node_node, &mut previous_node);
            } else {
                //falls noch nicht das Target erreicht wurde:
                //fuer den zu untersuchenden Nodes muessen alle Nachbarn gecheckt werden
                self.check_neighbours(
                    node_node,
                    node_weight,
                    &mut total_weight,
                    &mut previous_node,
                    &mut unvisited_nodes,
                );
            }
        }
        println!("Outer None");
        // Wenn keine mehr zu untersuchenden Nodes in der Queue sind bzw schon untersucht wurden oder gesperrt sind
        // folgt, dass kein Weg existiert
        None
    }

    pub fn smart_select(&mut self, start: Node) -> Option<ShortestPath> {
        println!("Smart select von Node: {:?}", start);
        //Dijkstra mit einer Menge an targets statt einem speziellen Target
        //Das erste Target was zurückgegeben wird ist automatisch das mit dem kürzesten Weg

        //visited_nodes: Nodes die der Dijkstra besucht hat
        let mut visited_nodes: Vec<Node> = Vec::new();
        // target_nodes sind alle möglichen Nodes welche einen Pfad zu sich selber mit Weight 0 haben (noch unexplored Direction)
        // als auch die, die noch nicht in scanned vorkommen (wurden bspw lediglich mit add_path() durch entdeckten Path hinzugefügt)
        let target_nodes: HashSet<Node> = self.target_nodes().0;
        // Bei den nodes mit einer unexplored Direction muss diese Direction auch gespeichert werden
        // (beim ShortestPath als letzte Direction vom target_node noch hinzufügen)
        let target_node_direction: HashMap<Node, Direction> = self.target_nodes().1;

        println!("Mögliche Target Nodes {:?}", target_nodes);

        // HashMap die jeder Node ihren vorherigen Node zuordnet des aktuell am kuerzesten bekanntesten path und die Direction
        // vom Vorgaenger zu dieser Node
        let mut previous_node: HashMap<Node, (Node, Direction)> = HashMap::new();
        // HashMap die jeder Node das weight des aktuell kuerzesten bekannten Pfades zuordnet
        let mut total_weight: HashMap<Node, Weight> = HashMap::new();
        // Binary Heap aus allen noch nicht besuchten Nodes (PriorityQueue)
        let mut unvisited_nodes: BinaryHeap<NodeInQueue> = BinaryHeap::new();

        //Startnode bekommt die Entfernung 0, alle anderen Nodes die Entfernung unendlich (Maximum value)
        self.initialize_dijkstra(start, &mut total_weight, &mut unvisited_nodes);

        //Queue die immer das Node mit dem kleinsten weight als erstes waehlt und bearbeitet,
        // node_node ist die Node, die gerade bearbeitet wird
        while let Some(NodeInQueue {
            weight: node_weight,
            node: node_node,
        }) = unvisited_nodes.pop()
        {
            //checken ob Node schonmal bearbeitet wurde, falls ja -> ueberspringen
            if visited_nodes.contains(&node_node) {
                continue;
            }
            // fals noch nicht bearbeitet, jetzt in schon besuchte Nodes hinzufuegen
            visited_nodes.push(node_node);
            // falls die zu untersuchende Node in den target_nodes ist => Algorithmus hat Ziel gefunden
            // der erste Node der gefunden wurde ist automatisch der mit dem kuerzesten Weg
            if target_nodes.contains(&node_node) {
                return self.found_target_ss(
                    start,
                    node_node,
                    target_node_direction,
                    previous_node,
                );
            }
            //falls noch nicht das Target erreicht wurde:
            else {
                self.check_neighbours(
                    node_node,
                    node_weight,
                    &mut total_weight,
                    &mut previous_node,
                    &mut unvisited_nodes,
                );
            }
        }
        None
    }

    fn check_neighbours(
        &self,
        node: Node,
        node_weight: Weight,
        total_weight: &mut HashMap<Node, Weight>,
        previous_node: &mut HashMap<Node, (Node, Direction)>,
        unvisited_nodes: &mut BinaryHeap<NodeInQueue>,
    ) {
        let all_paths = self.get_paths();

        for a_direction in self.all_directions().iter() {
            let neighbour_data = match all_paths.get(&node).unwrap().get(a_direction) {
                Some(n) => n,
                None => continue,
            };
            // falls an der zu untersuchenden Himmelsrichtung kein Node existiert (kein Nachbar): ueberspringen
            // Node des Nachbarn festlegen
            let neighbour_node: Node = neighbour_data.0;
            // Gewicht des Nachbarn festlegen
            let neighbour_weight: Weight = neighbour_data.2;
            // neues Gewicht berechnen (altes bis zum jetzigen Zeitpunkt bekannte Gewicht + neues Gewicht)
            let new_weight = node_weight + neighbour_weight;

            //falls nachbar existiert aber Pfad gesperrt ist oder es eine unerkundete Direction ist:
            // diesen Nachbar ignorieren
            if neighbour_weight == -1 || neighbour_weight == 0 {
                continue;
            }
            // nur falls das neue Gewicht kleiner ist, als das zurzeit minimal bekannte Gewicht: Pfad anpassen
            if new_weight < *total_weight.get(&neighbour_node).unwrap() {
                //Gewicht des Pfades anpassen
                total_weight.insert(neighbour_node, new_weight);
                //Vorgaenger anpassen
                previous_node.insert(neighbour_node, (node, *a_direction));
                //Nachbar als noch zu untersuchendes Element in die Queue einfuegen
                unvisited_nodes.push(NodeInQueue {
                    weight: new_weight,
                    node: neighbour_node,
                });
            }
        }
    }

    fn found_target_ss(
        &self,
        start: Node,
        target: Node,
        target_node_direction: HashMap<Node, Direction>,
        previous_node: HashMap<Node, (Node, Direction)>,
    ) -> Option<ShortestPath> {
        // found target smart select
        println!("Nächstes Target: {:?}", target);
        let mut last_direction = Direction::North;

        if target_node_direction.contains_key(&target) {
            last_direction = *target_node_direction.get(&target).unwrap();
        }
        let target_node = target;
        //Loesung aus bekannten Werten konstruieren
        //Vektor muss von hinten nach vorne aufgebaut werden und dann gespiegelt werden
        let mut reverse_shortest_existing_path: Vec<(Node, Direction)> = Vec::new();
        //hinten anfangen, also die zu erst untersuchende Node ist die target node:
        let mut investigating_node = target_node;

        //fuer jede zu untersuchende Node muessen folgende Schritte gemacht werden
        while investigating_node != start {
            // Vorgaenger durch HashMap bestimmen
            let my_previous_node = previous_node.get(&investigating_node).unwrap().0;
            // Direction vom Vorgaenger bestimmen
            let my_direction_from_previous_node = previous_node.get(&investigating_node).unwrap().1;
            // Tuple aus Vorgaenger und Direction in den Vektor einfuegen
            reverse_shortest_existing_path
                .push((my_previous_node, my_direction_from_previous_node));
            // neu zu untersuchende Node ist jetzt der Vorgaenger
            investigating_node = my_previous_node;
            // wird wiederholt bis man beim Start angekommen ist
        }
        // richtige Loesung aufbauen durch spiegeln des reverse Vektors
        let mut shortest_existing_path: Vec<(Node, Direction)> = Vec::new();
        for tuple in reverse_shortest_existing_path.into_iter().rev() {
            shortest_existing_path.push(tuple);
        }

        // falls von der letzten Node noch eine Direction bekannt ist diese Node mit Direction noch hinzufügen
        if target_node_direction.contains_key(&target) {
            shortest_existing_path.push((target, last_direction));
        }
        println!("Shortest Path zu Target: {:?}", shortest_existing_path);

        Some(shortest_existing_path)
    }

    fn found_target_sp(
        &self,
        start: Node,
        target: Node,
        previous_node: &mut HashMap<Node, (Node, Direction)>,
    ) -> Option<ShortestPath> {
        // found target shortest path
        //Loesung aus bekannten Werten konstruieren
        //Vektor muss von hinten nach vorne aufgebaut werden und dann gespiegelt werden
        let mut reverse_shortest_existing_path: ShortestPath = Vec::new();
        //hinten anfangen, also die zu erst untersuchende Node ist die target node:
        let mut investigating_node = target;

        //fuer jede zu untersuchende Node muessen folgende Schritte gemacht werden
        while investigating_node != start {
            // Vorgaenger durch HashMap bestimmen
            let my_previous_node = previous_node.get(&investigating_node).unwrap().0;
            // Direction vom Vorgaenger bestimmen
            let my_direction_from_previous_node = previous_node.get(&investigating_node).unwrap().1;
            // Tuple aus Vorgaenger und Direction in den Vektor einfuegen
            reverse_shortest_existing_path
                .push((my_previous_node, my_direction_from_previous_node));
            // neu zu untersuchende Node ist jetzt der Vorgaenger
            investigating_node = my_previous_node;
            // wird wiederholt bis man beim Start angekommen ist
        }
        // richtige Loesung aufbauen durch spiegeln des reverse Vektors
        let mut shortest_existing_path: ShortestPath = Vec::new();
        for tuple in reverse_shortest_existing_path.into_iter().rev() {
            shortest_existing_path.push(tuple);
        }
        println!("Shortest Path: {:?}", shortest_existing_path);
        Some(shortest_existing_path)
    }

    fn initialize_dijkstra(
        &self,
        start: Node,
        weight: &mut HashMap<Node, Weight>,
        unvisited: &mut BinaryHeap<NodeInQueue>,
    ) {
        //Startnode bekommt die Entfernung 0, alle anderen Nodes die Entfernung unendlich (Maximum value)
        for a_node in self.all_nodes() {
            if a_node == start {
                weight.insert(start, 0);
                unvisited.push(NodeInQueue {
                    weight: 0,
                    node: start,
                });
            } else {
                weight.insert(a_node, i64::MAX);
            }
        }
    }

    fn all_directions(&self) -> Vec<Direction> {
        vec![
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ]
    }

    fn all_nodes(&self) -> Vec<Node> {
        let all_paths = self.get_paths();
        let mut all_nodes: Vec<Node> = Vec::new();

        for path in all_paths.iter() {
            if all_nodes.contains(path.0) {
                continue;
            } else {
                all_nodes.push(*path.0);
            }
        }
        all_nodes
    }

    fn target_nodes(&self) -> (HashSet<Node>, HashMap<Node, Direction>) {
        let all_directions = self.all_directions();
        let all_nodes = self.all_nodes();
        let mut target_nodes = HashSet::new();
        let mut target_node_direction = HashMap::new();
        for node in all_nodes.iter() {
            // falls scanned nicht die jeweilige node enthält => zu targets hinzufügen
            if !self.scanned.contains(node) {
                target_nodes.insert(*node);
            }
            // danach vom jeweiligen Node in alle Directions schauen ob ein Pfad mit Weight 0 existiert
            // Diese Direction und den jeweiligen Node speichern
            for direction in all_directions.iter() {
                if let Some(inner_hm) = self.paths.get(node) {
                    if inner_hm.get(direction).is_none() {
                        continue;
                    }
                    match inner_hm.get(direction).unwrap() {
                        (a_node, _, 0) => {
                            let a_node = *a_node;
                            let copy_node = a_node;
                            target_nodes.insert(a_node);
                            target_node_direction.insert(copy_node, *direction);
                        }
                        _ => continue,
                    }
                }
            }
        }
        (target_nodes, target_node_direction)
    }

    fn add_unexplored(&mut self, node: Node, direction: Direction) {
        // schaeun ob schon eine innere HashMap existiert falls nicht eine erstellen und Werte einfügen, ansonsten anpassen
        // Ein Path mit weight !=0 kann einen Path mit Weight 0 überschreiben aber ein Path mit Weight 0 niemals einen Path mit Wert!=0
        // (wichtig falls Nodes doppelt gescannt werden)
        match self.paths.get(&node) {
            None => {
                let mut inner_hm: OutgoingPaths = HashMap::new();
                inner_hm.insert(direction, (node, direction, 0));
                self.paths.insert(node, inner_hm);
            }
            Some(inner) => {
                if inner.get(&direction).is_none() {
                    let mut inner_hm0: OutgoingPaths = self.paths.get(&node).unwrap().clone();
                    inner_hm0.insert(direction, (node, direction, 0));
                    self.paths.insert(node, inner_hm0);
                } else {
                    if let Some((_x, _y, z)) = inner.get(&direction)
                        && *z == 0
                    {
                        let mut inner_hm0: OutgoingPaths = self.paths.get(&node).unwrap().clone();
                        inner_hm0.insert(direction, (node, direction, 0));
                        self.paths.insert(node, inner_hm0);
                    }
                }
            }
        }
    }

    fn add_normal_path(
        &mut self,
        start_node: Node,
        start_direction: Direction,
        target_node: Node,
        target_direction: Direction,
        weight: Weight,
    ) {
        // da ein Bidirektionaler Pfad existiert auch von target zu start einen path einfeugen
        // immer jeweils innere und äußere HashMap anpassen
        if !self.paths.contains_key(&start_node) {
            let mut inner_hm1: HashMap<Direction, (Node, Direction, Weight)> = HashMap::new();
            inner_hm1.insert(start_direction, (target_node, target_direction, weight));
            self.paths.insert(start_node, inner_hm1);
        } else {
            let mut inner_hm1: HashMap<Direction, (Node, Direction, Weight)> =
                self.paths.get(&start_node).unwrap().clone();
            inner_hm1.insert(start_direction, (target_node, target_direction, weight));
            self.paths.insert(start_node, inner_hm1);
        }
        //analog zu dem Path: start => target
        if !self.paths.contains_key(&target_node) {
            let mut inner_hm2: HashMap<Direction, (Node, Direction, Weight)> = HashMap::new();
            inner_hm2.insert(target_direction, (start_node, start_direction, weight));
            self.paths.insert(target_node, inner_hm2);
        } else {
            let mut inner_hm2: HashMap<Direction, (Node, Direction, Weight)> =
                self.paths.get(&target_node).unwrap().clone();
            inner_hm2.insert(target_direction, (start_node, start_direction, weight));
            self.paths.insert(target_node, inner_hm2);
        }
    }
}
