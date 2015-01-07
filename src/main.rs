extern crate xml;

use std::os;

use std::io::{File, BufferedReader};
use std::fmt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::Thread;
use std::io::timer::sleep;
use std::time::duration::Duration;

use xml::reader::EventReader;
use xml::reader::events::XmlEvent;

fn main() {
	println!("XGConsole:");
	for arg in parse_command_line(os::args()).iter() {
		println!("  {}", arg);
	}
	sample();

	let (tx_result, rx_result): (Sender<String>, Receiver<String>) = channel();
	let (tx_task, rx_task): (Sender<String>, Receiver<String>) = channel();
	let mutex_rx_task = Arc::new(Mutex::new(rx_task));

	for cpu_id in range(0, std::os::num_cpus()) {
		let local_rx_task = mutex_rx_task.clone();
		let local_tx_result = tx_result .clone();
				Thread::spawn(move || {
				loop {
					let message: String;
					{
						match local_rx_task.lock().recv_opt() {
								Ok(v) => {message = v;}
								Err(_) => {break;}
							}
					}
					println!("{}: {}", cpu_id, message);
					sleep(Duration::milliseconds(100));
					local_tx_result.send(format!("Done {}", message));
				}
				println!("{}: done", cpu_id);
			}).detach();
	}
	free(tx_result);

	for task_id in range (0i, 50i) {
			tx_task.send(format!("Task {}", task_id));
	}
	free(tx_task);

	for message in rx_result.iter() {
		println!("B: {}", message);
	}
	println!("done");
}

fn free<T>(_:T) {
}

fn parse_command_line(args: Vec<String>) -> Vec<String> {
	let mut result: Vec<String> = Vec::new();
	for arg in args.slice(1, args.len()).iter() {
			result.push(arg.clone());
	}
	result
}

struct XgTask {
id: String,
title: Option<String>,
tool: String,
working_dir: String,
depends_on: Vec<String>,
}

impl fmt::Show for XgTask {
fn fmt(& self, f: &mut fmt::Formatter) -> fmt::Result {
	write!(f, "id={}, title={}, tool={}, working_dir={}, depends_on={}", self .id, self .title, self .tool, self .working_dir, self .depends_on)
}
}

struct XgTool {
id: String,
path: String,
params: String,
}

impl fmt::Show for XgTool {
fn fmt(& self, f: &mut fmt::Formatter) -> fmt::Result {
	write!(f, "id={}, path={}", self .id, self .path)
}
}

fn sample() {
	let file = File::open(&Path::new("tests/graph-parser.xml")).unwrap();
	let reader = BufferedReader::new(file);

	let mut parser = EventReader::new(reader);
	let mut tasks:Vec<XgTask> = vec![];
	let mut tools:HashMap<String, XgTool> = HashMap::new();
	for e in parser.events() {
		match e {
				XmlEvent::StartElement {name, attributes, ..} => {
				match name.local_name.as_slice() {
						"Task" =>
						{
							match xg_parse_task(&attributes) {
									Ok(task) =>
									{
											tasks.push(task);
									}
									Err(msg) =>
									{
										panic!(msg);
									}
								};
						}
						"Tool" =>
						{
							match xg_parse_tool(&attributes) {
									Ok(tool) =>
									{
											tools.insert(tool.id.to_string(), tool);
									}
									Err(msg) =>
									{
										panic!(msg);
									}
								};
						}
						_ => {}
					}
			}
				XmlEvent::EndElement{..} => {
			}
				_ => {
			}
			}
	}
}

fn map_attributes (attributes: &Vec<xml::attribute::OwnedAttribute>) -> HashMap< String, String> {
	let mut attrs: HashMap<String, String> = HashMap::new();
	for attr in attributes.iter() {
			attrs.insert(attr.name.local_name.clone(), attr.value.clone());
	}
	attrs
}

fn xg_parse_task (attributes: & Vec<xml::attribute::OwnedAttribute>)->Result<XgTask, String> {
	let mut attrs = map_attributes(attributes);
	// Name
	let id: String;
	match attrs.remove("Name") {
			Some(v) => {id = v;}
			_ => {return Err("Invalid task data: attribute @Name not found.".to_string());}
		}
	// Tool
	let tool: String;
	match attrs.remove("Tool") {
			Some(v) => {tool = v;}
			_ => {return Err("Invalid task data: attribute @Tool not found.".to_string());}
		}
	// WorkingDir
	let working_dir: String;
	match attrs.remove("WorkingDir") {
			Some(v) => {working_dir = v;}
			_ => {return Err("Invalid task data: attribute @WorkingDir not found.".to_string());}
		}
	// DependsOn
	let mut depends_on : Vec<String> = vec![];
	match attrs.remove("DependsOn") {
			Some(v) =>
			{
				for item in v.split_str(";").collect::<Vec<&str>>().iter() {
						depends_on.push(item.to_string())
				}
			}
			_ =>
			{
			}
		};

		Ok(XgTask {
	id: id.to_string(),
	title: attrs.remove("Caption"),
	tool: tool,
	working_dir: working_dir,
	depends_on: depends_on,
	})
}
fn xg_parse_tool (attributes: &Vec<xml::attribute::OwnedAttribute>)->Result<XgTool, String> {
	let mut attrs = map_attributes(attributes);
	// Name
	let id: String;
	match attrs.remove("Name") {
			Some(v) => {id = v;}
			_ => {return Err("Invalid task data: attribute @Name not found.".to_string());}
		}
	// Path
	let path: String;
	match attrs.remove("Path") {
			Some(v) => {path = v;}
			_ => {return Err("Invalid task data: attribute @Name not found.".to_string());}
		}

	Ok(XgTool {
	id: id,
	path: path,
	params: match attrs.remove("Params") {
			Some(v) => {v}
			_ => {"".to_string()}
		},
	})
}
