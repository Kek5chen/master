use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::thread::{JoinHandle, Thread};

struct SharedPhiloData {
    time_to_die: u32,
    time_to_eat: u32,
    time_to_sleep: u32,
    nb_meals: u32
}

struct PhiloData {
    meals_eaten: u32
}

impl Default for PhiloData {
    fn default() -> Self {
        PhiloData {
            meals_eaten: 0
        }
    }
}

pub struct Simulation {
    nb_philo: u32,
    shared_data: Arc<SharedPhiloData>,
    owned_data: Vec<Arc<PhiloData>>,
}

impl Simulation {
    pub fn init(nb_philo: u32, time_to_die: u32, time_to_eat: u32, time_to_sleep: u32, nb_meals: u32) -> Self {
        Simulation {
            nb_philo,
            shared_data: Arc::new(SharedPhiloData {
                time_to_die,
                time_to_eat,
                time_to_sleep,
                nb_meals
            }),
            owned_data: Vec::new()
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut philos: Vec<JoinHandle<u32>> = Vec::with_capacity(self.nb_philo as usize);

        self.owned_data.reserve(self.nb_philo as usize);
        for _ in 0..self.nb_philo {
            let philo_data = Arc::new(PhiloData::default());
            self.owned_data.push(philo_data.clone());
            philos.push(Philosopher::create(self.shared_data.clone(), philo_data));
        }
        Ok(())
    }
}

pub struct Philosopher {
    sim_data: Arc<SharedPhiloData>,
}

impl Philosopher {
    fn create(sim_data: Arc<SharedPhiloData>, philo_data: Arc<PhiloData>) -> JoinHandle<u32> {
        thread::spawn(move || {
            let mut philo = Philosopher {
                sim_data
            };
            philo.live()
        })
    }

    fn live(&self) -> u32 {
        0
    }
}