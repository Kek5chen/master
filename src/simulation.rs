use std::error::Error;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize};
use std::sync::atomic::Ordering::*;
use std::thread;
use std::thread::{current, JoinHandle, sleep, Thread};
use std::time::{Duration, Instant};

struct SharedPhiloData {
    time_to_die: u32,
    time_to_eat: u32,
    time_to_sleep: u32,
    nb_meals: u32,
    start_time: RwLock<Instant>,
    done: AtomicBool,
}

struct PhiloData {
    num: AtomicUsize,
    meals_eaten: AtomicU32,
    last_eat: RwLock<Instant>,
    is_eating: RwLock<bool>,
    forks: [Arc<Mutex<()>>; 2],
}

impl PhiloData {
    fn new(num: usize, left_fork: Arc<Mutex<()>>, right_fork: Arc<Mutex<()>>) -> PhiloData {
        PhiloData {
            num: AtomicUsize::new(num),
            meals_eaten: AtomicU32::new(0),
            last_eat: RwLock::new(Instant::now()),
            is_eating: RwLock::new(false),
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
                start_time: RwLock::new(Instant::now()),
                done: AtomicBool::new(false),
            }),
            owned_data: Vec::new(),
            forks,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Starting simulation..");
        let mut philos: Vec<(Arc<Philosopher>, JoinHandle<Result<(), ()>>)> = Vec::with_capacity(self.nb_philo as usize);

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

        while !self.shared_data.done.load(Acquire) {
            for (philo, _) in &philos {
                let ms_since_last_eat = philo.data.last_eat.read().unwrap().elapsed().as_millis();

                if ms_since_last_eat > self.shared_data.time_to_die as u128 && !*philo.data.is_eating.read().unwrap() {
                    self.shared_data.done.store(true, SeqCst);

                    let time = self.shared_data.start_time.read().unwrap().elapsed().as_millis();
                    Philosopher::print_manual(PhiloAction::Die, time, philo.data.num.load(Relaxed));
                }
            }
        }
        for (_, handle) in philos {
            handle.join();
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
    Die,
    Finish,
}

struct Philosopher {
    sim_data: Arc<SharedPhiloData>,
    data: Arc<PhiloData>,
}

impl<'a> Philosopher {
    fn create(sim_data: Arc<SharedPhiloData>, philo_data: Arc<PhiloData>) -> (Arc<Philosopher>, JoinHandle<Result<(), ()>>) {
        let philo = Arc::new(Philosopher {
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

    fn print_manual(action: PhiloAction, time: u128, num: usize) {
        let action_text = match action {
            PhiloAction::PickLeftFork => "picked up the left fork",
            PhiloAction::PickRightFork => "picked up the right fork",
            PhiloAction::Eat => "started eating",
            PhiloAction::DropLeftFork => "dropped the left fork",
            PhiloAction::DropRightFork => "dropped the right fork",
            PhiloAction::Sleep => "fell asleep",
            PhiloAction::WakeUp => "woke up",
            PhiloAction::Die => "died",
            PhiloAction::Finish => "finished eating."
        };
        println!("{}\t\tphilo [{}]  {}", time, num, action_text)
    }

    fn print_action(&self, action: PhiloAction) -> Result<(), ()>{
        if self.sim_data.done.load(Acquire) {
            return Err(());
        }
        let current_time = self.sim_data.start_time.read().unwrap().elapsed();
        Self::print_manual(action, current_time.as_millis(), self.data.num.load(Relaxed));
        Ok(())
    }

    fn live(&self) -> Result<(), ()>{
        *self.data.last_eat.write().unwrap() = Instant::now();
        for _ in 0..self.sim_data.nb_meals {
            if self.sim_data.done.load(Acquire) {
                return Ok(());
            }
            // pick up forks
            let lock = self.data.forks[0].lock();
            self.print_action(PhiloAction::PickLeftFork)?;
            let lock2 = self.data.forks[1].lock();
            self.print_action(PhiloAction::PickRightFork)?;

            // eat
            self.print_action(PhiloAction::Eat)?;
            *self.data.is_eating.write().unwrap() = true;
            self.data.meals_eaten.fetch_add(1, Relaxed);
            sleep(Duration::from_millis(self.sim_data.time_to_eat as u64));
            *self.data.is_eating.write().unwrap() = false;
            *self.data.last_eat.write().unwrap() = Instant::now();

            // drop forks
            drop(lock);
            self.print_action(PhiloAction::DropLeftFork)?;
            drop(lock2);
            self.print_action(PhiloAction::DropRightFork)?;

            // sleep
            self.print_action(PhiloAction::Sleep)?;
            sleep(Duration::from_millis(self.sim_data.time_to_sleep as u64));
            self.print_action(PhiloAction::WakeUp)?;
        }
        self.print_action(PhiloAction::Finish)?;
        self.sim_data.done.store(true, Release);
        Ok(())
    }
}