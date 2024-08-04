mod threadpool;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use clap::Parser;
use threadpool::ThreadPool;
use types::{Color, Square};
use xq::{
    generate::{find_magic, ChessMove},
    rng::Rng,
    *,
};

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
            "\"{}_magics\" : [",
            slider_name,
        );
        for square in start_range {
            let slider = Arc::clone(&slider);
            let rng = Arc::clone(&self.rng);
            self.pool.execute(move || {
                FindMagicsWorker::find_and_print_step(slider, square, rng);
            });
        }
        self.pool.wait();
        println!("],");
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
            let rook = rook();
            let rook = Arc::new(rook);
            worker.find_and_print_all_magics(rook, "rook");
        }),
    );
    tasks_manage.insert(
        "CANNON",
        Box::new(|worker: &mut FindMagicsWorker| {
            let cannon = cannon();
            let cannon = Arc::new(cannon);
            worker.find_and_print_all_magics(cannon, "cannon");
        }),
    );
    tasks_manage.insert(
        "KNIGHT",
        Box::new(|worker: &mut FindMagicsWorker| {
            let knight = knight();
            let knight = Arc::new(knight);
            worker.find_and_print_all_magics(knight, "knight");
        }),
    );
    tasks_manage.insert(
        "BISHOP",
        Box::new(|worker: &mut FindMagicsWorker| {
            let bishop = bishop();
            let bishop = Arc::new(bishop);
            worker.find_and_print_all_magics(bishop, "bishop");
        }),
    );
    tasks_manage.insert(
        "RED_PAWN",
        Box::new(|worker: &mut FindMagicsWorker| {
            let pawn  = pawn(Color::Red);
            let pawn = Arc::new(pawn);
            worker.find_and_print_all_magics(pawn, "red_pawn");
        }),
    );
    tasks_manage.insert(
        "BLACK_PAWN",
        Box::new(|worker: &mut FindMagicsWorker| {
            let pawn = pawn(Color::Black);
            let pawn = Arc::new(pawn);
            worker.find_and_print_all_magics(pawn, "black_pawn");
        }),
    );
    tasks_manage.insert(
        "ADVISOR",
        Box::new(|worker: &mut FindMagicsWorker| {
            let advisor = advisor();
            let advisor = Arc::new(advisor);
            worker.find_and_print_all_magics(advisor, "advisor");
        }),
    );
    tasks_manage.insert(
        "KING",
        Box::new(|worker: &mut FindMagicsWorker| {
            let king = king();
            let king = Arc::new(king);
            worker.find_and_print_all_magics(king, "king");
        }),
    );
    println!("{{");
    tasks_manage.run(task)?;
    println!("}}");
    Ok(())
}
