use std::cell::Cell;
use std::error::Error;
use std::ops::Sub;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicU32, AtomicUsize};
use std::sync::atomic::Ordering::*;
use std::thread;
use std::thread::{current, JoinHandle, sleep, Thread};
use std::time::{Duration, Instant};

struct SharedPhiloData {
    time_to_die: u32,
    time_to_eat: u32,
    time_to_sleep: u32,
    nb_meals: u32,
    start_time: RwLock<Instant>
}

struct PhiloData {
    num: AtomicUsize,
    meals_eaten: AtomicU32,
    last_eat: Mutex<Instant>,
    forks: [Arc<Mutex<()>>; 2]
}

impl PhiloData {
    fn new(num: usize, left_fork: Arc<Mutex<()>>, right_fork: Arc<Mutex<()>>) -> PhiloData {
        PhiloData {
            num: AtomicUsize::new(num),
            meals_eaten: AtomicU32::new(0),
            last_eat: Mutex::new(Instant::now()),
            forks: [left_fork, right_fork]
        }
    }
}

pub struct Simulation {
    nb_philo: u32,
    shared_data: Arc<SharedPhiloData>,
    owned_data: Vec<Arc<PhiloData>>,
    forks: Vec<Arc<Mutex<()>>>,
}

impl Simulation {
    pub fn init(nb_philo: u32, time_to_die: u32, time_to_eat: u32, time_to_sleep: u32, nb_meals: u32) -> Self {
        let forks = (0..nb_philo).map(|_| Arc::new(Mutex::new(()))).collect();

        Simulation {
            nb_philo,
            shared_data: Arc::new(SharedPhiloData {
                time_to_die,
                time_to_eat,
                time_to_sleep,
                nb_meals,
                start_time: RwLock::new(Instant::now())
            }),
            owned_data: Vec::new(),
            forks,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Starting simulation..");
        let mut philos: Vec<(Arc<Philosopher>, JoinHandle<()>)> = Vec::with_capacity(self.nb_philo as usize);

        self.owned_data.reserve(self.nb_philo as usize);
        *self.shared_data.start_time.write().unwrap() = Instant::now();
        for i in 0usize..self.nb_philo as usize {
            let left_fork = self.forks[i].clone();
            let right_fork = self.forks[(i + 1) % self.forks.len()].clone();

            let philo_data = Arc::new(PhiloData::new(i, left_fork, right_fork));
            self.owned_data.push(philo_data.clone());

            let philo = Philosopher::create(self.shared_data.clone(), philo_data);
            philos.push(philo);
        }

        for philo in philos {
            philo.1.join().expect("Could not join thread.");
        }
        Ok(())
    }
}

enum PhiloAction {
    PickLeftFork,
    PickRightFork,
    Eat,
    DropLeftFork,
    DropRightFork,
    Sleep,
    WakeUp,
}

struct Philosopher {
    sim_data: Arc<SharedPhiloData>,
    data: Arc<PhiloData>,
}

impl<'a> Philosopher {
    fn create(sim_data: Arc<SharedPhiloData>, philo_data: Arc<PhiloData>) -> (Arc<Philosopher>, JoinHandle<()>) {
        let mut philo = Arc::new(Philosopher {
            sim_data,
            data: philo_data,
        });

        let philo_clone = philo.clone();
        let handle = thread::Builder::new()
            .name(format!("Philosopher {}", philo.data.num.load(Relaxed)))
            .spawn(move || {
                philo_clone.live()
        }).unwrap();


        (philo, handle)
    }

    fn print_action(&self, action: PhiloAction) {
        let current_time = self.sim_data.start_time.read().unwrap().elapsed();
        let action_text = match action {
            PhiloAction::PickLeftFork => "picked up the left fork",
            PhiloAction::PickRightFork => "picked up the right fork",
            PhiloAction::Eat => "started eating",
            PhiloAction::DropLeftFork => "dropped the left fork",
            PhiloAction::DropRightFork => "dropped the right fork",
            PhiloAction::Sleep => "fell asleep",
            PhiloAction::WakeUp => "woke up",
        };

        println!("{}\t\tphilo [{}]  {}", current_time.as_millis(), self.data.num.load(Relaxed), action_text)
    }

    fn live(&self) {
        *self.data.last_eat.lock().unwrap() = Instant::now();
        for i in 0..self.sim_data.nb_meals {
            // pick up forks
            let lock = self.data.forks[0].lock();
            self.print_action(PhiloAction::PickLeftFork);
            let lock2 = self.data.forks[1].lock();
            self.print_action(PhiloAction::PickRightFork);

            // eat
            self.print_action(PhiloAction::Eat);
            sleep(Duration::from_millis(self.sim_data.time_to_eat as u64));
            *self.data.last_eat.lock().unwrap() = Instant::now();
            self.data.meals_eaten.fetch_add(1, Relaxed);

            // drop forks
            drop(lock);
            self.print_action(PhiloAction::DropLeftFork);
            drop(lock2);
            self.print_action(PhiloAction::DropRightFork);

            // sleep
            self.print_action(PhiloAction::Sleep);
            sleep(Duration::from_millis(self.sim_data.time_to_sleep as u64));
        }
    }
}