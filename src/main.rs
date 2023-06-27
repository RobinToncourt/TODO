use std::env;
use rusqlite::Connection;
use std::process;
use std::error::Error;
// use settings::Settings;
use iced::{widget::{button, column, text, Column, scrollable, container}, Application, Command, Theme, Element, Settings, Length, futures::future::ok};

// const
const DATABASE_FILE_PATH: &str = "tasks.sqlite3";
const HELP: &str = "help";
const GUI: &str = "gui";
const LIST: &str = "list";
const SHOW: &str = "show";
const ADD: &str = "add";
const ALL: &str = "all";
const TODO: &str = "todo";
const DONE: &str = "done";
const DROP: &str = "drop";
// const DELETE: &str = "delete";

// Error messages
// const UNFOUND_DATABASE: &str = "Tasks database not found at location: ";
const AT_LEAST_ONE_PARAMETER: &str = "Need at least 1 parameter, please use the 'help' action.";
const PARAMETER_NOT_A_NUMBER: &str = "is not a number.";
const FAILED_PARSE_STR_TO_U32: &str = "The second parameter has to be a number.";
// const ACTION_NEEDS_TWO_PARAMETERS: &str = "This action needs two parameters.";
const UNKNOWN_ACTION: &str = "Unknown action, please refer to the 'help' action.";
const UNABLE_TO_ADD_TASK: &str = "Unable to add task.";
const INVALID_TASK_NUMBER: &str = "Invalid task number";
const UNKNOWN_STATE: &str = "Etat inconnu.";

// SQL queries
const SQL_QUERY_SELECT_ALL_TASKS: &str = "SELECT id, task, state FROM tasks;";
const SQL_QUERY_SELECT_TODO_TASKS: &str = "SELECT id, task FROM tasks WHERE state=1;";
const SQL_QUERY_SELECT_DONE_TASKS: &str = "SELECT id, task FROM tasks WHERE state=2;";
const SQL_QUERY_SELECT_DROP_TASKS: &str = "SELECT id, task FROM tasks WHERE state=3;";
const SQL_QUERY_ADD: &str = "INSERT INTO tasks(task) VALUES(?1);";
const SQL_QUERY_DONE: &str = "UPDATE tasks SET state=2 WHERE id=?1;";
const SQL_QUERY_DROP: &str = "UPDATE tasks SET state=3 WHERE id=?1;";
// const SQL_QUERY_DELETE: &str = "DELETE FROM tasks WHERE id=?1;";

/*
 * Structs.
 */
#[derive(Debug, Clone, Copy)]
enum State {
    All,
    Todo,
    Done,
    Dropped,
}

struct Task {
    id: i32,
    task: String,
    state: State
}

impl Task {
    fn print(&self) -> String {
        format!("{} {}", self.id, self.task)
    }
}

#[derive(Debug, Clone)]
enum Action<'a> {
    Help,
    Gui,
    List(State),
    Add(&'a str),
    Done(u32),
    Drop(u32),
    // Delete(u32),
}

struct Todo<'a> {
    name: &'a str,
    conn: Connection,
    tasks: Vec<Task>,
}

impl<'a> Application for Todo<'a> {
    type Message = Action<'a>;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = (Connection, Vec<Task>);
    
    fn new(flags: Self::Flags) -> (Self, Command<Action<'a>>) {
        ( Todo{ name: "Todo", conn: flags.0, tasks: flags.1 }, Command::none() )
    }
    
    fn title(&self) -> String {
        String::from(self.name)
    }
    
    fn view(&self) -> Element<Action<'a>> {
        
        
        let content = column![
            button("Ajouter").on_press(Action::Add("Nouvelle tâche.")),
            button("Fini").on_press(Action::Done(42)),
            button("Abandonné").on_press(Action::Drop(42)),
        ];
        scrollable(
            container(content)
                .width(Length::Fill)
                .padding(40)
                .center_x(),
        ).into()
    }
    
    fn update(&mut self, action: Action) -> Command<Action<'a>> {
        
        match action {
            Action::Add(task) => println!("Ajouter {}.", task),
            Action::Done(id) => println!("Fini tache {}.", id),
            Action::Drop(id) => println!("Tache {} abandonné...", id),
            _ => println!("ça marche ? ça marche !"),
        }
        
        Command::none()
    }
}

/*
 * Main functions.
 */
fn main() {
    let args: Vec<String> = env::args().collect();
    
    let conn: Connection = open_database(DATABASE_FILE_PATH)
        .unwrap_or_else(|err| {
            println!("{}", err);
            process::exit(0);
    });
    
    let action = parse_actions(&args)
        .unwrap_or_else(|err| {
            println!("{}", err);
            process::exit(0);
    });
    
    if let Err(e) = do_action(conn, action) {
        println!("{}", e);
    }
}

// Open the database and return the connection.
fn open_database(file_path: &str) -> Result<Connection, Box<dyn Error>> {
    let conn = Connection::open(&file_path)?;
    Ok(conn)
}

// Parse the user input action.
fn parse_actions(args: &[String]) -> Result<Action, String> {
    
    if args.len() < 2 {
        return Err(AT_LEAST_ONE_PARAMETER.to_string());
    }
    
    let arg1 = args[1].as_str();
    let mut arg2: Option<&str> = Option::None;
    
    if args.len() >= 3 {
        arg2 = Option::Some(args[2].as_str());
    }
    
    // arg2 can be a State, String, u32.
    
    let action = match arg1 {
        HELP => Action::Help,
        GUI => Action::Gui,
        LIST | SHOW => {
            
            let state = match arg2 {
                Some(s) => get_state(s),
                None => Ok(State::Todo),
            };
            
            match state {
                Ok(s) => Action::List(s),
                Err(error_message) => return Err(error_message),
            }
        },
        ADD => {
            match arg2 {
                Some(s) => Action::Add(s),
                None => return Err(AT_LEAST_ONE_PARAMETER.to_string()),
            }
        },
        DONE => {
            let str_u32 = match arg2 {
                Some(s) => s,
                None => return Err(AT_LEAST_ONE_PARAMETER.to_string()),
            };
            
            match parse_str_to_u32(str_u32) {
                Ok(id) => Action::Done(id),
                Err(e) => return Err(e),
            }
        },
        DROP => {
            let str_u32 = match arg2 {
                Some(s) => s,
                None => return Err(AT_LEAST_ONE_PARAMETER.to_string()),
            };
            
            match parse_str_to_u32(str_u32) {
                Ok(id) => Action::Drop(id),
                Err(e) => return Err(e),
            }
        },
        // DELETE => Action::Delete(),
        _ => return Err(UNKNOWN_ACTION.to_string()),
    };
    
    Ok(action)
}

fn get_state(s: &str) -> Result<State, String> {
    match s {
        ALL => Ok(State::All),
        TODO => Ok(State::Todo),
        DONE => Ok(State::Done),
        DROP => Ok(State::Dropped),
        _ => Err(UNKNOWN_STATE.to_string())
    }
}

fn do_action(conn: Connection, action: Action) -> Result<(), String> {
    match action {
        Action::Help => help(),
        Action::Gui => gui(conn),
        Action::List(state) => list(&conn, state),
        Action::Add(task) => add(&conn, task),
        Action::Done(task_id) => done(&conn, task_id),
        Action::Drop(task_id) => drop(&conn, task_id),
        // Action::Delete(task_id) => delete(&conn, task_id),
    }
}

// Parse a &str to an u32.
fn parse_str_to_u32(str: &str) -> Result<u32, String> {
    match str.parse::<u32>() {
        Ok(id) => Ok(id),
        Err(_) => Err(format!("{} {}", str, PARAMETER_NOT_A_NUMBER))
    }
}

/*
 * User actions.
 */
fn gui(conn: Connection) -> Result<(), String> {
    println!("Running GUI version...");
    
    let tasks = get_tasks(&conn);
    
    let iced_run_result =
        Todo::run( Settings {
            id: Option::None,
            window: iced::window::Settings::default(),
            flags: (conn, tasks),
            default_font: Option::None,
            default_text_size: 20.0,
            text_multithreading: false,
            antialiasing: false,
            exit_on_close_request: true,
            try_opengles_first: false
        });
    match iced_run_result {
        Ok(_) => { println!("All good."); }
        Err(_) => { println!("An error occured."); }
    }
    
    Ok(())
}

// Prints the help command.
fn help() -> Result<(), String> {
    println!("Options
    help: print this message
    list: list all TODO state tasks
    show: same as list
    add \"Task\": add the task
    done [task number]: set the task to done state
    drop [task number]: set the task to dropped state");
    // delete [task number]: delete the task associated with the number (showed by list)
    
    Ok(())
}

fn get_tasks(conn: &Connection) -> Vec<Task> {
    let mut stmt = conn.prepare(SQL_QUERY_SELECT_ALL_TASKS).unwrap();
    
    let mapped_rows = stmt.query_map((), |row| {
        let id = row.get(0)?;
        let task = row.get(1)?;
        let state = match row.get(3)? {
            1 => State::Todo,
            2 => State::Done,
            3 => State::Dropped,
            _ => State::Todo,
        };
        
        Ok(Task { id, task, state })
    }).unwrap();
    
    let mut tasks = Vec::new();
    
    for task in mapped_rows {
        tasks.push(task.unwrap());
    }
    
    tasks
}

// List all the tasks.
fn list(conn: &Connection, state: State) -> Result<(), String> {
    let mut stmt = match state {
        State::All => conn.prepare(SQL_QUERY_SELECT_ALL_TASKS).unwrap(),
        State::Todo => conn.prepare(SQL_QUERY_SELECT_TODO_TASKS).unwrap(),
        State::Done => conn.prepare(SQL_QUERY_SELECT_DONE_TASKS).unwrap(),
        State::Dropped => conn.prepare(SQL_QUERY_SELECT_DROP_TASKS).unwrap(),
    };
    
    // Retreive the tasks
    let tasks = stmt.query_map((), |row| {
        let id = row.get(0)?;
        let task = row.get(1)?;
        
        Ok(Task { id, task, state: State::Todo })
    }).unwrap();
    
    // Print tasks.
    for task in tasks {
        println!("{}", task.unwrap().print());
    }
    
    Ok(())
}

// Add a new task.
fn add(conn: &Connection, task: &str) -> Result<(), String> {
    let result =
        conn.execute(SQL_QUERY_ADD, [task]);
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(UNABLE_TO_ADD_TASK.to_string())
    }
}

// Set task state to done.
fn done(conn: &Connection, task_id: u32) -> Result<(), String> {
    let result =
        conn.execute(SQL_QUERY_DONE, [task_id]);
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(INVALID_TASK_NUMBER.to_string())
    }
}

// Set task state to done.
fn drop(conn: &Connection, task_id: u32) -> Result<(), String> {
    let result =
        conn.execute(SQL_QUERY_DROP, [task_id]);
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(INVALID_TASK_NUMBER.to_string())
    }
}

// Delete a task.
// fn delete(conn: &Connection, task_id: u32) -> Result<(), String> {
//     let result =
//         conn.execute(SQL_QUERY_DELETE, [task_id]);
//     match result {
//         Ok(_) => Ok(()),
//         Err(_) => Err(INVALID_TASK_NUMBER.to_string())
//     }
// }
