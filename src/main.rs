mod simulation;

use std::env;
use std::error::Error;
use simulation::Simulation;

fn init_simulation(args: &Vec<String>) -> Result<Simulation, Box<dyn Error>> {
    let nb_philo: u32 = args[1]
        .parse()
        .map_err(|_| "Could not parse nb_philo")?;
    let time_to_die: u32 = args[2]
        .parse()
        .map_err(|_| "Could not parse time_to_die")?;
    let time_to_eat: u32 = args[3]
        .parse()
        .map_err(|_| "Could not parse time_to_eat")?;
    let time_to_sleep: u32 = args[4]
        .parse()
        .map_err(|_| "Could not parse time_to_sleep")?;
    let nb_meals: u32 = match args.get(5) {
        Some(nb_meals) => nb_meals.parse()?,
        None => u32::MAX,
    };

    Ok(Simulation::init(nb_philo, time_to_die, time_to_eat, time_to_sleep, nb_meals))
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    if args.len() != 5 && args.len() != 6 {
        println!("Usage: {} <nb_philo> <time_to_die> <time_to_eat> <time_to_sleep> [nb_meals]",
                 args.first().unwrap());
        return;
    }

    let mut simulation = match init_simulation(&mut args) {
        Ok(sim) => sim,
        Err(e) => {
            eprintln!("The parameters you supplied were invalid. {}", e);
            return;
        }
    };

    match simulation.run() {
        Ok(()) => println!("Simulation end. Success."),
        Err(e) => println!("Simulation end. Failure. ({})", e),
    }
}
