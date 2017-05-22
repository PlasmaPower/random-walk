use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate clap;

extern crate rand;
use rand::Rng;

pub struct Room {
    pub doors: Vec<&'static str>,
}

impl Room {
    pub fn new(doors: Vec<&'static str>) -> Room {
        Room {
            doors: doors,
        }
    }

    pub fn random_door<R: Rng>(&self, rng: &mut R) -> &'static str {
        rng.choose(&self.doors).unwrap()
    }
}

lazy_static! {
    static ref ROOMS: HashMap<&'static str, Room> = {
        let mut map = HashMap::new();
        map.insert("A", Room::new(vec!["F", "D"]));
        map.insert("B", Room::new(vec!["D"]));
        map.insert("C", Room::new(vec!["EXIT"]));
        map.insert("D", Room::new(vec!["A", "B", "C", "E"]));
        map.insert("E", Room::new(vec!["D", "F"]));
        map.insert("F", Room::new(vec!["EXIT"]));
        map
    };
}

fn main() {
    let args = load_yaml!("../cli.yml");
    let args = clap::App::from_yaml(args).get_matches();
    let mut rng = rand::weak_rng();
    let n: usize = args.value_of("number").unwrap().parse().expect("Expected number of walks to be a positive integer");
    let starting_room = ROOMS.get(args.value_of("starting_room").unwrap()).expect("Expected starting room to be a room");
    let mut hours_sum: u64 = 0;
    let mut hours_squared_sum: u64 = 0;
    for _ in 0..n {
        let mut hours = 0;
        let mut room = starting_room;
        loop {
            hours += 1;
            let next_room = room.random_door(&mut rng);
            if next_room == "EXIT" {
                break;
            }
            room = ROOMS.get(next_room).unwrap();
        }
        hours_sum += hours;
        hours_squared_sum += hours * hours;
    }
    let mean = (hours_sum as f64) / (n as f64);
    println!("mean: {}", mean);
    println!("stdev: {}", ((hours_squared_sum as f64) / ((n - 1) as f64) - mean.powi(2)).sqrt());
}
