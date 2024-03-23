use std::error::Error;

pub struct Simulation {
    nb_philo: u32,
    time_to_die: u32,
    time_to_eat: u32,
    time_to_sleep: u32,
    nb_meals: u32
}

impl Simulation {
    pub fn init(nb_philo: u32, time_to_die: u32, time_to_eat: u32, time_to_sleep: u32, nb_meals: u32) -> Self {
        Simulation {
            nb_philo,
            time_to_die,
            time_to_eat,
            time_to_sleep,
            nb_meals
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}