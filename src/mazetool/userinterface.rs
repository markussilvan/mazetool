use crossbeam::channel::{Receiver, Sender};

use super::common::{ Job, UIRequest };

/// Trait for features required from a Mazetool user interface
pub trait UserInterface
{
	fn new(tx: Sender<Job>, rx: Receiver<UIRequest>) -> Self;
	fn run(&mut self);
}
