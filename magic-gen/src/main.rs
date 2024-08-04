mod cannon;
mod generate;
mod knight;
mod pawn;
mod rng;
mod rook;
mod threadpool;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cannon::CannonAttack;
use clap::Parser;
use generate::{find_magic, ChessMove};
use knight::{
    LameLeaper, BISHOP_DELTAS, BISHOP_LAMELS, BISHOP_START_RANGE, KNIGHT_DELTAS, KNIGHT_LAMELS,
};
use pawn::{Pawn, ToNeighbour, BLACK_PAWN, RED_PAWN};
use rng::Rng;
use rook::{Slider, SLIDER_ONE_STEP};
use threadpool::ThreadPool;
use types::{BitBoard, Square};

struct FindMagicsWorker {
    pool: ThreadPool,
    rng: Arc<Mutex<Rng>>,
}

impl FindMagicsWorker {
    fn new(size: usize) -> Self {
        FindMagicsWorker {
            pool: ThreadPool::new(size),
            rng: Arc::new(Mutex::new(Rng::default())),
        }
    }

    fn find_and_print_all_magics(
        &mut self,
        slider: Arc<dyn ChessMove + Send + Sync + 'static>,
        slider_name: &str,
    ) {
        let start_range = slider.start_range();
        println!(
            "pub const {}_MAGICS: &[MagicEntry; {}] = &[",
            slider_name,
            start_range.len()
        );
        for square in start_range {
            let slider = Arc::clone(&slider);
            let rng = Arc::clone(&self.rng);
            self.pool.execute(move || {
                FindMagicsWorker::find_and_print_step(slider, square, rng);
            });
        }
        self.pool.wait();
        println!("];");
    }

    fn find_and_print_step(
        slider: Arc<dyn ChessMove + Send + Sync + 'static>,
        square: Square,
        rng: Arc<Mutex<Rng>>,
    ) {
        let index_bits = slider.relevant_blockers(square).popcnt() as u8;
        let (entry, _) = find_magic(&*slider, square, index_bits, rng);
        // In the final move generator, each table is concatenated into one contiguous table
        // for convenience, so an offset is added to denote the start of each segment.
        println!("    {},", entry);
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional task_name to operate on
    task_name: Option<String>,
    /// number of thread
    #[arg(short, long, value_name = "thread_count")]
    thread_count: Option<usize>,
}

enum TasksOption {
    Task(String),
    All,
    Nothing,
}

struct TasksManage<'a> {
    worker: FindMagicsWorker,
    tasks: HashMap<String, Box<dyn Fn(&mut FindMagicsWorker) -> () + 'a>>,
}

#[derive(Debug)]
enum TasksFinishWithErr {
    TaskNoFound,
    DoNoThing,
}

impl<'a> TasksManage<'a> {
    fn new(worker: FindMagicsWorker) -> Self {
        TasksManage {
            worker,
            tasks: HashMap::new(),
        }
    }

    fn insert(&mut self, name: &str, task: Box<dyn Fn(&mut FindMagicsWorker) -> () + 'a>) {
        let name = name.to_uppercase();
        self.tasks.insert(name, task);
    }

    fn run(&mut self, tasks_option: TasksOption) -> Result<(), TasksFinishWithErr> {
        match tasks_option {
            TasksOption::Task(name) => {
                if let Some(task) = self.tasks.get_mut(&name) {
                    task(&mut self.worker);
                    Ok(())
                } else {
                    Err(TasksFinishWithErr::TaskNoFound)
                }
            }
            TasksOption::All => {
                if self.tasks.is_empty() {
                    Err(TasksFinishWithErr::DoNoThing)
                } else {
                    for task in self.tasks.values_mut() {
                        task(&mut self.worker)
                    }
                    Ok(())
                }
            }
            TasksOption::Nothing => Err(TasksFinishWithErr::DoNoThing),
        }
    }
}

fn main() -> Result<(), TasksFinishWithErr> {
    let cli = Cli::parse();
    let task_name = cli.task_name.as_deref();
    let worker = if let Some(thread_count) = cli.thread_count {
        FindMagicsWorker::new(thread_count)
    } else {
        FindMagicsWorker::new(1)
    };
    let task = match task_name {
        Some(name) if name != "none" => TasksOption::Task(name.to_uppercase()),
        Some(_) => TasksOption::Nothing,
        None => TasksOption::All,
    };
    let mut tasks_manage = TasksManage::new(worker);
    tasks_manage.insert(
        "ROOK",
        Box::new(|worker: &mut FindMagicsWorker| {
            let rook = Slider::new(SLIDER_ONE_STEP, Vec::from(Square::ALL));
            let rook = Arc::new(rook);
            worker.find_and_print_all_magics(rook, "ROOK");
        }),
    );
    tasks_manage.insert(
        "CANNON",
        Box::new(|worker: &mut FindMagicsWorker| {
            let cannon = CannonAttack::new();
            let cannon = Arc::new(cannon);
            worker.find_and_print_all_magics(cannon, "CANNON");
        }),
    );
    tasks_manage.insert(
        "KNIGHT",
        Box::new(|worker: &mut FindMagicsWorker| {
            let knight = LameLeaper::new(KNIGHT_DELTAS, KNIGHT_LAMELS, Vec::from(Square::ALL));
            let knight = Arc::new(knight);
            worker.find_and_print_all_magics(knight, "KNIGHT");
        }),
    );
    tasks_manage.insert(
        "BISHOP",
        Box::new(|worker: &mut FindMagicsWorker| {
            let start_range = Vec::from(BISHOP_START_RANGE);
            let bishop = LameLeaper::new(BISHOP_DELTAS, BISHOP_LAMELS, start_range);
            let bishop = Arc::new(bishop);
            worker.find_and_print_all_magics(bishop, "BISHOP");
        }),
    );
    tasks_manage.insert(
        "RED_PAWN",
        Box::new(|worker: &mut FindMagicsWorker| {
            let pawn = ToNeighbour::pawn(Pawn {
                begin: 3,
                end: 9,
                offset: 0,
                range: RED_PAWN,
                end_mask: BitBoard(0x1ff << 81),
            });
            let pawn = Arc::new(pawn);
            worker.find_and_print_all_magics(pawn, "RED_PAWN");
        }),
    );
    tasks_manage.insert(
        "BLACK_PAWN",
        Box::new(|worker: &mut FindMagicsWorker| {
            let pawn = ToNeighbour::pawn(Pawn {
                begin: 6,
                end: 0,
                offset: 1,
                range: BLACK_PAWN,
                end_mask: BitBoard(0x1ff),
            });
            let pawn = Arc::new(pawn);
            worker.find_and_print_all_magics(pawn, "BLACK_PAWN");
        }),
    );
    tasks_manage.insert(
        "ADVISOR",
        Box::new(|worker: &mut FindMagicsWorker| {
            let advisor = ToNeighbour::advisor();
            let advisor = Arc::new(advisor);
            worker.find_and_print_all_magics(advisor, "ADVISOR");
        }),
    );
    tasks_manage.insert(
        "KING",
        Box::new(|worker: &mut FindMagicsWorker| {
            let king = ToNeighbour::king();
            let king = Arc::new(king);
            worker.find_and_print_all_magics(king, "KING");
        }),
    );
    tasks_manage.run(task)
}
