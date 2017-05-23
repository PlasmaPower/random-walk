use std::thread;
use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate clap;

extern crate rand;
use rand::Rng;

extern crate fnv;
use fnv::FnvHashMap;

pub struct ThreadResult {
    sum: u64,
    sum_of_squares: u64,
    counts: Option<FnvHashMap<u8, u64>>,
}

pub struct Room {
    doors: Vec<&'static str>,
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

fn random_walk(mut rng: &mut Rng, starting_room: &Room) -> u8 {
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
    hours
}

fn main() {
    let args = load_yaml!("../cli.yml");
    let args = clap::App::from_yaml(args).get_matches();
    let n: usize = args.value_of("number").unwrap().parse().expect("Expected number of walks to be a positive integer");
    let thread_count: usize = args.value_of("threads").unwrap().parse().expect("Expected number of threads to be a positive integer");
    let starting_room = ROOMS.get(args.value_of("starting_room").unwrap()).expect("Expected starting room to be a room");
    let outputs = args.values_of("outputs").unwrap().collect::<Vec<_>>();
    let output_raw = outputs.contains(&"raw");
    let output_counts = outputs.contains(&"counts");
    let mut threads = Vec::new();
    for i in 0..thread_count {
        threads.push(thread::spawn(move || {
            let mut rng = rand::weak_rng();
            let mut our_sum: u64 = 0;
            let mut our_squared_sum: u64 = 0;
            let mut our_count = n / thread_count;
            if i == thread_count - 1 {
                our_count += n % thread_count;
            }
            let mut counts = if output_counts {
                Some(FnvHashMap::default())
            } else {
                None
            };
            for _ in 0..our_count {
                let hours = random_walk(&mut rng, starting_room);
                if output_raw {
                    println!("{}", hours);
                }
                if let Some(counts) = counts.as_mut() {
                    *counts.entry(hours).or_insert(0) += 1;
                }
                let hours = hours as u64;
                our_sum += hours;
                our_squared_sum += hours * hours;
            }
            ThreadResult {
                sum: our_sum,
                sum_of_squares: our_squared_sum,
                counts: counts,
            }
        }));
    }
    let mut hours_sum: u64 = 0;
    let mut hours_squared_sum: u64 = 0;
    let mut counts = FnvHashMap::default();
    for thread in threads {
        let result = thread.join().unwrap();
        hours_sum += result.sum;
        hours_squared_sum += result.sum_of_squares;
        if let Some(result_counts) = result.counts {
            for (hours, n) in result_counts {
                *counts.entry(hours).or_insert(0) += n;
            }
        }
    }
    if outputs.contains(&"mean") {
        println!("mean: {}", (hours_sum as f64) / (n as f64));
    }
    if outputs.contains(&"stdev") {
        println!("stdev: {}", (((hours_squared_sum as f64) - ((hours_sum * hours_sum) as f64) / (n as f64)) / ((n - 1) as f64)).sqrt());
    }
    if output_counts {
        let min_hours = counts.keys().min().cloned().unwrap_or(0);
        let max_hours = counts.keys().max().cloned().unwrap_or(0);
        for time in min_hours..(max_hours + 1) {
            println!("{:2}: {}", time, counts.get(&time).unwrap_or(&0));
        }
    }
}
