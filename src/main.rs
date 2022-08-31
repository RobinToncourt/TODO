use std::env;
use rusqlite::Connection;
use std::process;
use std::error::Error;

// const
const DATABASE_FILE_PATH: &str = "tasks.sqlite3";
const HELP: &str = "help";
const LIST: &str = "list";
const SHOW: &str = "show";
const ADD: &str = "add";
const DONE: &str = "done";
const DROP: &str = "drop";
const DELETE: &str = "delete";

// Error messages
//const UNFOUND_DATABASE: &str = "Tasks database not found at location: ";
const AT_LEAST_ONE_PARAMETER: &str = "Need at least 1 parameter.";
const PARAMETER_NOT_A_NUMBER: &str = "is not a number.";
const FAILED_PARSE_STR_TO_U32: &str = "The second parameter has to be a number.";
const ACTION_NEEDS_TWO_PARAMETERS: &str = "This action needs two parameters.";
const UNKNOWN_ACTION: &str = "Unknown action, please refer to the 'help' action.";
const UNABLE_TO_ADD_TASK: &str = "Unable to add task.";
const INVALID_TASK_NUMBER: &str = "Invalid task number";

// SQL queries
const SQL_QUERY_SELECT_ALL_TODO: &str = "SELECT id, task FROM tasks WHERE state=1;";
const SQL_QUERY_ADD: &str = "INSERT INTO tasks(task) VALUES(?1);";
const SQL_QUERY_DONE: &str = "UPDATE tasks SET state=2 WHERE id=?1;";
const SQL_QUERY_DROP: &str = "UPDATE tasks SET state=3 WHERE id=?1;";
const SQL_QUERY_DELETE: &str = "DELETE FROM tasks WHERE id=?1;";

struct Options<'a> {
    action: &'a str,
    arg2: Option<&'a str>
}

struct Task {
    id: i32,
    task: String,
}

impl Task {
    fn print(&self) -> String {
        format!("{} {}", self.id, self.task)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let conn = open_database(DATABASE_FILE_PATH)
        .unwrap_or_else(|err| {
            println!("{}", err);
            process::exit(0);
    });
    let options = parse_options(&args)
        .unwrap_or_else(|err| {
            println!("{}", err);
            process::exit(0);
    });
    
    if let Err(e) = execute_action(conn, options) {
        println!("{}", e);
    }
}

// Open the database and return the connection.
fn open_database(file_path: &str) -> Result<Connection, Box<dyn Error>> {
    let conn = Connection::open(&file_path)?;
    Ok(conn)
    
    /* match conn {
        Ok(conn) => Ok(conn),
        Err(_) => Err(format!("{} {}.", UNFOUND_DATABASE, &file_path))
    } */
}

// Parse the parameters 
fn parse_options(args: &[String]) -> Result<Options, String> {
    if args.len() < 2 {
        return Err(AT_LEAST_ONE_PARAMETER.to_string());
    }
    
    let action = &args[1];
    let mut arg2 = Option::None;
    
    if args.len() >= 3 {
        arg2 = Option::Some(args[2].as_str());
    }
    
    Ok(Options { action, arg2 })
}

// Executes the user specified action
fn execute_action(conn: Connection, options: Options) -> Result<(), String> {
    let action = options.action;
    
    match action {
        HELP => help(),
        LIST | SHOW => list(&conn),
        _ => {
            if let Some(task) = options.arg2 {
                if action == ADD {
                    add(&conn, task)
                }                
                else if let Ok(task_id) = parse_str_to_u32(task) {
                    match action {
                        DONE => {
                        done(&conn, task_id)
                        },
                        DROP => {
                            drop(&conn, task_id)
                        },
                        DELETE => {
                            delete(&conn, task_id)
                        }
                        _ => {
                            Err(UNKNOWN_ACTION.to_string())
                        }
                    }
                }
                else {
                    Err(FAILED_PARSE_STR_TO_U32.to_string())
                }
            }
            else {
                Err(ACTION_NEEDS_TWO_PARAMETERS.to_string())
            }
        }
    }
}

// Parse a &str to an u32.
fn parse_str_to_u32(str: &str) -> Result<u32, String> {
    match str.parse::<u32>() {
        Ok(id) => Ok(id),
        Err(_) => Err(format!("{} {}", str, PARAMETER_NOT_A_NUMBER))
    }
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

// List all the tasks.
fn list(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn.prepare(SQL_QUERY_SELECT_ALL_TODO).unwrap();
    
    // Retreive the tasks
    let tasks = stmt.query_map((), |row| {
        let id = row.get(0)?;
        let task: String = row.get(1)?;
        
        Ok(Task { id, task })
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
fn delete(conn: &Connection, task_id: u32) -> Result<(), String> {
    let result =
        conn.execute(SQL_QUERY_DELETE, [task_id]);
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(INVALID_TASK_NUMBER.to_string())
    }
}
