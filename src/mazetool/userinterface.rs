use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;

use mazetool::common::{ Job, UIRequest };

/// Trait for features required from a Mazetool user interface
pub trait UserInterface
{
	fn new() -> Self;
	fn parse_args(&self, tx: &Sender<Job>) -> bool;
	fn run(tx: Sender<Job>, rx: Receiver<UIRequest>) -> thread::JoinHandle<()>;
}
