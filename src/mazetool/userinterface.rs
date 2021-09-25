use crossbeam::channel::{Receiver, Sender};

use super::common::{ Job, UIRequest };

/// Trait for features required from a Mazetool user interface
pub trait UserInterface
{
	fn new(tx: Sender<Job>, rx: Receiver<UIRequest>) -> Self;
	fn parse_args(&self, tx: &Sender<Job>) -> bool;
	fn run(&mut self);
}
