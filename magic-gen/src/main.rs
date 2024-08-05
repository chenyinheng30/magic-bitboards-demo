mod threadpool;

use clap::Parser;
use generate::MagicEntryGen;
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
};
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
    ) -> Vec<MagicEntryGen> {
        let start_range = slider.start_range();
        let mut table = Vec::new();
        let receiver = {
            let (sender, receiver) = mpsc::channel();
            for square in start_range {
                let slider = Arc::clone(&slider);
                let rng = Arc::clone(&self.rng);
                let sender = sender.clone();
                self.pool.execute(move || {
                    let g = FindMagicsWorker::find_and_print_step(slider, square, rng);
                    sender.send(g).unwrap();
                });
            }
            receiver
        };
        for g in receiver {
            table.push(g);
        }
        table
    }

    fn find_and_print_step(
        slider: Arc<dyn ChessMove + Send + Sync + 'static>,
        square: Square,
        rng: Arc<Mutex<Rng>>,
    ) -> MagicEntryGen {
        let index_bits = slider.relevant_blockers(square).popcnt() as u8;
        let (entry, _) = find_magic(&*slider, square, index_bits, rng);
        // In the final move generator, each table is concatenated into one contiguous table
        // for convenience, so an offset is added to denote the start of each segment.
        return entry;
    }
}

enum TasksOption {
    Task(String),
    All,
    Nothing,
}

struct TasksManage<'a> {
    worker: FindMagicsWorker,
    tasks: HashMap<String, Box<dyn Fn(&mut FindMagicsWorker) -> Vec<MagicEntryGen> + 'a>>,
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

    fn insert(
        &mut self,
        name: &str,
        task: Box<dyn Fn(&mut FindMagicsWorker) -> Vec<MagicEntryGen> + 'a>,
    ) {
        let name = name.to_lowercase();
        self.tasks.insert(name, task);
    }

    fn run(
        &mut self,
        tasks_option: TasksOption,
    ) -> Result<HashMap<String, Vec<MagicEntryGen>>, TasksFinishWithErr> {
        let mut tables = HashMap::<String, Vec<MagicEntryGen>>::new();
        match tasks_option {
            TasksOption::Task(name) => {
                let name = name.to_lowercase();
                if let Some(task) = self.tasks.get_mut(&name) {
                    let table = task(&mut self.worker);
                    let name = format!("{}_magic_table", name);
                    tables.insert(name, table);
                    Ok(tables)
                } else {
                    Err(TasksFinishWithErr::TaskNoFound)
                }
            }
            TasksOption::All => {
                if self.tasks.is_empty() {
                    Err(TasksFinishWithErr::DoNoThing)
                } else {
                    for (name, task) in self.tasks.iter() {
                        let table = task(&mut self.worker);
                        let name = format!("{}_magic_table", name);
                        tables.insert(name, table);
                    }
                    Ok(tables)
                }
            }
            TasksOption::Nothing => Err(TasksFinishWithErr::DoNoThing),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional task_name to operate on
    task_name: Option<String>,
    /// number of thread
    #[arg(short, long, value_name = "N_JOBS")]
    jobs: Option<usize>,
}

fn main() -> Result<(), TasksFinishWithErr> {
    let cli = Cli::parse();
    let task_name = cli.task_name.as_deref();
    let worker = if let Some(thread_count) = cli.jobs {
        FindMagicsWorker::new(thread_count)
    } else {
        FindMagicsWorker::new(1)
    };
    let task = match task_name {
        Some(name) if name != "none" => TasksOption::Task(name.to_string()),
        Some(_) => TasksOption::Nothing,
        None => TasksOption::All,
    };
    let mut tasks_manage = TasksManage::new(worker);
    tasks_manage.insert(
        "ROOK",
        Box::new(|worker: &mut FindMagicsWorker| {
            let rook = rook();
            let rook = Arc::new(rook);
            worker.find_and_print_all_magics(rook)
        }),
    );
    tasks_manage.insert(
        "CANNON",
        Box::new(|worker: &mut FindMagicsWorker| {
            let cannon = cannon();
            let cannon = Arc::new(cannon);
            worker.find_and_print_all_magics(cannon)
        }),
    );
    tasks_manage.insert(
        "KNIGHT",
        Box::new(|worker: &mut FindMagicsWorker| {
            let knight = knight();
            let knight = Arc::new(knight);
            worker.find_and_print_all_magics(knight)
        }),
    );
    tasks_manage.insert(
        "BISHOP",
        Box::new(|worker: &mut FindMagicsWorker| {
            let bishop = bishop();
            let bishop = Arc::new(bishop);
            worker.find_and_print_all_magics(bishop)
        }),
    );
    tasks_manage.insert(
        "RED_PAWN",
        Box::new(|worker: &mut FindMagicsWorker| {
            let pawn = pawn(Color::Red);
            let pawn = Arc::new(pawn);
            worker.find_and_print_all_magics(pawn)
        }),
    );
    tasks_manage.insert(
        "BLACK_PAWN",
        Box::new(|worker: &mut FindMagicsWorker| {
            let pawn = pawn(Color::Black);
            let pawn = Arc::new(pawn);
            worker.find_and_print_all_magics(pawn)
        }),
    );
    tasks_manage.insert(
        "ADVISOR",
        Box::new(|worker: &mut FindMagicsWorker| {
            let advisor = advisor();
            let advisor = Arc::new(advisor);
            worker.find_and_print_all_magics(advisor)
        }),
    );
    tasks_manage.insert(
        "KING",
        Box::new(|worker: &mut FindMagicsWorker| {
            let king = king();
            let king = Arc::new(king);
            worker.find_and_print_all_magics(king)
        }),
    );
    let tables = tasks_manage.run(task)?;
    println!("{}", serde_json::to_string(&tables).unwrap());
    Ok(())
}
