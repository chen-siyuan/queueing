use rand_distr::{Distribution, Exp, LogNormal};

#[derive(Debug)]
enum Status {
    Idle,
    Busy(f64, u32),
}

#[derive(Debug)]
struct Queue<S: Distribution<f64>> {
    job_size: S,
    status: Status,
}

impl<S: Distribution<f64>> Queue<S> {
    fn new(job_size: S) -> Self {
        Queue {
            job_size,
            status: Status::Idle,
        }
    }

    fn time_until_completion(&self) -> Option<f64> {
        match self.status {
            Status::Idle => None,
            Status::Busy(time_until_completion, _) => Some(time_until_completion),
        }
    }

    fn num_waiting(&self) -> Option<u32> {
        match self.status {
            Status::Idle => None,
            Status::Busy(_, num_waiting) => Some(num_waiting),
        }
    }

    fn increment(&mut self) {
        self.status = match self.status {
            Status::Idle => Status::Busy(self.job_size.sample(&mut rand::thread_rng()), 0),
            Status::Busy(time_until_completion, num_waiting) => {
                Status::Busy(time_until_completion, num_waiting + 1)
            }
        }
    }

    fn elapse(&mut self, time: f64) {
        assert!(time > 0f64, "invalid time {}", time);
        self.status = match self.status {
            Status::Idle => Status::Idle,
            Status::Busy(time_until_completion, num_waiting) => {
                assert!(
                    time <= time_until_completion,
                    "invalid time {} for time_until_completion {}",
                    time,
                    time_until_completion
                );
                if time < time_until_completion {
                    Status::Busy(time_until_completion - time, num_waiting)
                } else {
                    match num_waiting {
                        0 => Status::Idle,
                        num_waiting => Status::Busy(
                            self.job_size.sample(&mut rand::thread_rng()),
                            num_waiting - 1,
                        ),
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct Simulation<I: Distribution<f64>, S: Distribution<f64>> {
    inter_arrival_time: I,
    clock: f64,
    count: u32,
    queue: Queue<S>,
    time_until_arrival: f64,
}

impl<I: Distribution<f64>, S: Distribution<f64>> Simulation<I, S> {
    fn new(inter_arrival_time: I, job_size: S) -> Self {
        let mut simulation = Simulation {
            inter_arrival_time,
            clock: 0f64,
            count: 0,
            queue: Queue::new(job_size),
            time_until_arrival: 0f64,
        };
        simulation.time_until_arrival = simulation
            .inter_arrival_time
            .sample(&mut rand::thread_rng());
        simulation
    }

    fn step(&mut self) {
        let time = match self.queue.time_until_completion() {
            None => self.time_until_arrival,
            Some(time_until_completion) => time_until_completion.min(self.time_until_arrival),
        };
        self.clock += time;
        self.queue.elapse(time);
        if self.time_until_arrival == time {
            self.queue.increment();
            self.time_until_arrival = self.inter_arrival_time.sample(&mut rand::thread_rng());
        } else {
            self.count += 1;
            self.time_until_arrival -= time
        }
    }
}

fn main() {
    let mut simulation = Simulation::new(
        Exp::new(1.0 / 7.0).unwrap(),
        LogNormal::new(1.5, 0.5).unwrap(),
    );
    for _ in 0..1_000 {
        println!("{}\t{}\t{:?}", simulation.clock, simulation.count, simulation.queue.num_waiting());
        simulation.step()
    }
}
