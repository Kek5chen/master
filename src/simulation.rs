use std::error::Error;
use std::sync::{Arc, Mutex};
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
        let mut philos: Vec<(Arc<Philosopher>, JoinHandle<u32>)> = Vec::with_capacity(self.nb_philo as usize);

        self.owned_data.reserve(self.nb_philo as usize);
        for i in 0..self.nb_philo {
            let philo_data = Arc::new(PhiloData::default());
            self.owned_data.push(philo_data.clone());
            let philo = Philosopher::create(i, self.shared_data.clone(), philo_data);
            philos.push(philo);
        }

        for philo in philos {
            philo.1.join().expect("Could not join thread.");
        }
        Ok(())
    }
}

pub struct Philosopher {
    sim_data: Arc<SharedPhiloData>,
    data: Arc<PhiloData>,
}

impl Philosopher {
    fn create(philo_num: u32, sim_data: Arc<SharedPhiloData>, philo_data: Arc<PhiloData>) -> (Arc<Philosopher>, JoinHandle<u32>) {
        let mut philo = Arc::new(Philosopher {
            sim_data,
            data: philo_data,
        });

        let philo_clone = philo.clone();
        let handle = thread::Builder::new()
            .name(format!("Philosopher {}", philo_num))
            .spawn(move || {
                philo_clone.live()
        }).unwrap();


        (philo, handle)
    }

    fn live(&self) -> u32 {
        println!("Henlo from {}", thread::current().name().unwrap_or("Philosopher X"));
        0
    }
}