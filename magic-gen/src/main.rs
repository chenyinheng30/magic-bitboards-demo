use clap::Parser;
use generate::MagicEntryGen;
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
};
use types::Square;
use xq::{
    generate::{find_magic, ChessMove},
    rng::Rng,
    *,
};

struct FindMagicsWorker {
    rng: Arc<Mutex<Rng>>,
}

impl FindMagicsWorker {
    fn new() -> Self {
        FindMagicsWorker {
            rng: Arc::new(Mutex::new(Rng::default())),
        }
    }

    fn find_and_print_all_magics(
        &mut self,
        slider: Arc<dyn ChessMove + Send + Sync + 'static>,
    ) -> Vec<MagicEntryGen> {
        let start_range = slider.possible_squares();
        let mut table = Vec::new();
        let receiver = {
            let (sender, receiver) = mpsc::channel();
            for square in start_range {
                let slider = Arc::clone(&slider);
                let rng = Arc::clone(&self.rng);
                let sender = sender.clone();
                rayon::spawn(move || {
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
enum Error {
    TaskNoFound,
    DoNoThing,
    ThreadPoolBuildError,
}

impl From<rayon::ThreadPoolBuildError> for Error {
    fn from(_: rayon::ThreadPoolBuildError) -> Self {
        Self::ThreadPoolBuildError
    }
}

impl<'a> TasksManage<'a> {
    fn new(worker: FindMagicsWorker) -> Self {
        TasksManage {
            worker,
            tasks: HashMap::new(),
        }
    }

    fn insert(&mut self, name: &str, task: Arc<dyn ChessMove + Send + Sync + 'static>) {
        let name = name.to_lowercase();
        self.tasks.insert(
            name,
            Box::new(move |worker: &mut FindMagicsWorker| {
                worker.find_and_print_all_magics(task.clone())
            }),
        );
    }

    fn run(
        &mut self,
        tasks_option: TasksOption,
    ) -> Result<HashMap<String, Vec<MagicEntryGen>>, Error> {
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
                    Err(Error::TaskNoFound)
                }
            }
            TasksOption::All => {
                if self.tasks.is_empty() {
                    Err(Error::DoNoThing)
                } else {
                    for (name, task) in self.tasks.iter() {
                        let table = task(&mut self.worker);
                        let name = format!("{}_magic_table", name);
                        tables.insert(name, table);
                    }
                    Ok(tables)
                }
            }
            TasksOption::Nothing => Err(Error::DoNoThing),
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

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let task_name = cli.task_name.as_deref();
    let worker = FindMagicsWorker::new();
    if let Some(thread_count) = cli.jobs {
        rayon::ThreadPoolBuilder::default()
            .num_threads(thread_count)
            .build()?;
    };
    let task = match task_name {
        Some(name) if name != "none" => TasksOption::Task(name.to_string()),
        Some(_) => TasksOption::Nothing,
        None => TasksOption::All,
    };
    let mut tasks_manage = TasksManage::new(worker);
    tasks_manage.insert("ROOK", Arc::new(rook()));
    tasks_manage.insert("CANNON", Arc::new(cannon()));
    tasks_manage.insert("KNIGHT", Arc::new(knight()));
    tasks_manage.insert("BISHOP", Arc::new(bishop()));
    tasks_manage.insert("KING", Arc::new(king()));
    let tables = tasks_manage.run(task)?;
    println!("{}", serde_json::to_string(&tables).unwrap());
    Ok(())
}
