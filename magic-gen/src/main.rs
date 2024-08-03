mod generate;
mod rng;
mod rook;
mod threadpool;

use std::{
    collections::HashMap, sync::{Arc, Mutex}
};

use clap::Parser;
use generate::*;
use rng::*;
use rook::Slider;
use threadpool::*;
use types::*;

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
        println!(
            "pub const {}_MAGICS: &[MagicEntry; Square::NUM] = &[",
            slider_name
        );
        let total_table_size = Arc::new(Mutex::new(0usize));
        for &square in &Square::ALL {
            let slider = Arc::clone(&slider);
            let rng = Arc::clone(&self.rng);
            let total_table_size = Arc::clone(&total_table_size);
            self.pool.execute(move || {
                FindMagicsWorker::find_and_print_step(slider, square, rng, total_table_size);
            });
        }
        self.pool.wait();
        println!("];");
        println!(
            "pub const {}_TABLE_SIZE: usage = {} KiB;",
            slider_name,
            *total_table_size.lock().unwrap() / 1024 * 16
        );
    }

    fn find_and_print_step(
        slider: Arc<dyn ChessMove + Send + Sync + 'static>,
        square: Square,
        rng: Arc<Mutex<Rng>>,
        total_table_size: Arc<Mutex<usize>>,
    ) {
        let index_bits = slider.relevant_blockers(square).popcnt() as u8;
        let (entry, table) = find_magic(&*slider, square, index_bits, rng);
        // In the final move generator, each table is concatenated into one contiguous table
        // for convenience, so an offset is added to denote the start of each segment.
        let mut total_table_size = total_table_size.lock().unwrap();
        println!(
            "    MagicEntry {{ mask: 0x{:016X}, magic: 0x{:032X}, shift: {}, offset: {} }},",
            entry.mask.0, entry.magic, entry.shift, total_table_size
        );
        *total_table_size += table.len();
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
    tasks: HashMap<String, Box<dyn FnMut() -> () + 'a>>,
}

#[derive(Debug)]
enum TasksFinishWithErr {
    TaskNoFound,
    DoNoThing,
}

impl<'a> TasksManage<'a> {
    fn new() -> Self {
        TasksManage {
            tasks: HashMap::new(),
        }
    }

    fn insert(&mut self, name: &str, task: Box<dyn FnMut() -> () + 'a>) {
        let name = name.to_uppercase();
        self.tasks.insert(name, task);
    }

    fn run(&mut self, tasks_option: TasksOption) -> Result<(), TasksFinishWithErr> {
        match tasks_option {
            TasksOption::Task(name) => {
                if let Some(task) = self.tasks.get_mut(&name) {
                    task();
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
                        task()
                    }
                    Ok(())
                }
            }
            TasksOption::Nothing => Err(TasksFinishWithErr::DoNoThing),
        }
    }
}

fn main() -> Result<(), TasksFinishWithErr>{
    let cli = Cli::parse();
    let task_name = cli.task_name.as_deref();
    let mut worker = if let Some(thread_count) = cli.thread_count {
        FindMagicsWorker::new(thread_count)
    } else {
        FindMagicsWorker::new(1)
    };
    let task = match task_name {
        Some(name) if name != "none" => TasksOption::Task(name.to_uppercase()),
        Some(_) => TasksOption::Nothing,
        None => TasksOption::All,
    };
    let mut tasks_manage = TasksManage::new();
    tasks_manage.insert("ROOK", Box::new(||{
        let rook = Slider::new([(1, 0), (0, 1), (-1, 0), (0, -1)]);
        let rook = Arc::new(rook);
        worker.find_and_print_all_magics(rook, "ROOK");
    }));
    tasks_manage.run(task)
}
