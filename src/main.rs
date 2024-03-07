#![windows_subsystem = "windows"]
use std::{env, fs, io};
use std::fs::{DirEntry, OpenOptions};
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process;
use std::time::SystemTime;
use chrono::prelude::*;
use regex::{Captures, Regex};
fn expand_variables(s:&String, log:&String, env:&Regex) -> String { // Expand environment variables
  let result: String = env.replace_all(s, |c: &Captures| match &c[1] {
    "" => String::from("%"),
    variable_name => env::var(variable_name).unwrap_or_else( |err| { message2log(&log, format!("env::var() error: {err}")); process::exit(1); } )
  }).into();
  return result;
}
fn date_stamp() -> String {
  let local: DateTime<Local> = DateTime::from(Utc::now());
  return format!("{}",local.format("%Y%m%d"));
}
fn message2log(file_name:&String, message: String) {
  let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(file_name)
    .unwrap();
  let local: DateTime<Local> = DateTime::from(Utc::now());
  if let Err(e) = writeln!(file, "{} {}", local.format("%Y%m%d%H%M%S"), message.trim()) {
    eprintln!("Couldn't write to file {} error:{}", file_name, e);
  }
}
fn lookup(file:&DirEntry,base_name:&str) -> bool { return file.file_name().to_str().unwrap().contains(base_name); }
fn prune(file:DirEntry,deleted:&mut i32) -> io::Result<()> {
  let now = SystemTime::now();
  const PRUNE_DAYS:u64 = 30*24*60*60;
  if now.duration_since(file.metadata().unwrap().created().unwrap()).ok().unwrap().as_secs() > PRUNE_DAYS {
    *deleted += 1;
    fs::remove_file(file.path())?;
  }
  Ok(())
}
fn remove_logs(log_file:&String, base_name:&str) {
  let files = fs::read_dir(env::var("TEMP").expect("$env:TEMP is not SET")).unwrap();
  let mut deleted:i32 = 0;
  files
    .filter_map(Result::ok)
    .filter(|file| lookup(file,base_name))
    .for_each(|file| { prune(file, &mut deleted).unwrap_or_else( |err| { message2log(&log_file, format!("Can't delete log: {err}")); process::exit(2); }); } );
  if deleted > 0 {
    message2log(log_file,format!("Removed {} old logs.",deleted));
  }
}
fn main() {
  let arguments: Vec<String> = env::args().collect();
  let base_name = &*Path::new(&arguments[0]).file_stem().expect("EMPTY").to_os_string().into_string().unwrap();
  let log_file = env::var("TEMP").expect("$env:TEMP is not SET") + "\\" + base_name + "-" + &date_stamp() + ".log";
  if arguments.len() < 3 {
    message2log(&log_file, String::from("Expecting more arguments {normal|hide} program-to-run arguments"));
    process::exit(3);
  }
  let environment: Regex = Regex::new("%([0-9A-Za-z_()]*)%").unwrap_or_else( |err| { message2log(&log_file, format!("Regex error: {err}")); process::exit(1); } );
  let exec = expand_variables(&arguments[2], &log_file, &environment);
  let window_type = &arguments[1];
  remove_logs(&log_file, base_name);
  const NO_WINDOW: u32 = 0x08000000;
  let mut new_process = process::Command::new(exec.clone());
  new_process.env(r"SEE_MASK_NOZONECHECKS", r"1");
  if window_type.eq_ignore_ascii_case(&String::from("hide")) { new_process.creation_flags(NO_WINDOW); }
  for arg in &arguments[3..] { new_process.arg(arg); }
  let mut line_arguments: String = "".to_owned();
  for argument in new_process.get_args().collect::<Vec<_>>().iter() {
    let mut parameter:String = argument.to_os_string().into_string().unwrap();
    if parameter.find(" ").is_some() { parameter = format!("{}{}{}", '"', parameter, '"'); }
    line_arguments = format!("{}{}{}", line_arguments, expand_variables(&parameter, &log_file, &environment), ' ');
  }
  if let Ok(child) = new_process.spawn() {
    message2log(&log_file, format!(r#"Start "{}" {} PID({})"#, exec, line_arguments.trim_end(), child.id()));
  } else {
    message2log(&log_file, format!(r#"Start "{}" {} failed!"#, exec, line_arguments.trim_end()));
  }
}
/*

Launch process with arguments, new process can spawn as hidden or with own new window. Spawned processes are logged into log file.

LOG:       Create log file in %TEMP%\{exe-base-name}-{date-time-stamp}.log
Arguments: 1st argument {hide|normal} - hidden or normal window for new process
           2nd process to start
           3rd ... last are passed to new process where all %env-variables% are expanded
Examples:

spawn hide "%ProgramFiles%\Powershell\7\pwsh.exe" -nologo -noprofile -file "%USERPROFILE%\somescript.ps1"

Will run script in background without any console window

spawn normal "%ProgramFiles%\Powershell\7\pwsh.exe" -nologo

Will run shell with console window

Author:   Eduard Švarc
Version:  1.0.3
Date:     7.3.2024
Revisions:

1.0.3 2024-03-07 Eduard Švarc
- Prune for logs older than 30 days

*/