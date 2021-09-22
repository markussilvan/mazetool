use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use mazetool::common::{ Job, UIRequest };

/// Trait for features required from a Mazetool user interface
pub trait UserInterface
{
	fn new(tx: Sender<Job>, rx: Receiver<UIRequest>) -> Self;
	fn parse_args(&self, tx: &Sender<Job>) -> bool;
	fn run(&self);
}
