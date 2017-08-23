#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate rppal;

use serde_json::value::Value;

use rppal::gpio::{Level, Mode, PullUpDown, GPIO};
use rppal::gpio::Error as GPIOError;

use std::thread::sleep;
use std::time::Duration;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::thread;
use std::sync::mpsc::{channel, Receiver};
use std::error::Error;

const BUTTON: u8 = 4;
const RELAY: u8 = 18;

const LED_GREEN: u8 = 5;
const LED_RED: u8 = 6;

pub struct Button {
	pin: u8,
	gpio: GPIO
}

impl Button {
	pub fn new(pin: u8) -> Result<Button, GPIOError> {
		match GPIO::new() {
			Ok(mut gpio) => {
				gpio.set_mode(pin, Mode::Input);
				gpio.set_pullupdown(pin, PullUpDown::PullDown);

				Ok(Button {
					pin: pin,
					gpio: gpio
				})
			}
			Err(e) => Err(e),
		}
	}


	pub fn poll<F>(&mut self, mut closure: F) -> Result<(), GPIOError>
	where
		F: FnMut(),
	{
		let mut button_state = match self.gpio.read(self.pin) {
			Ok(res) => res,
			Err(err) => return Err(err),
		};

		loop {
			match self.gpio.read(self.pin) {
				Ok(bs) => {
					if bs != button_state && bs == Level::High {
						closure();
					}

					button_state = bs;

					sleep(Duration::from_millis(50));
				}
				Err(err) => return Err(err),
			}
		}
		Ok(())
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
	cmd_type: String,
	// TODO: Find proper solution to convert HashMap to string
	payload: Value
}

#[derive(Serialize, Deserialize, Debug)]
struct PowerCommand {
	power: bool
}

#[derive(Serialize, Deserialize, Debug)]
struct PigeonState {
	operating_time: u64,
	release_time: u64
}

struct Pigeon {
	gpio: GPIO,
	power_cmd_rx: Receiver<PowerCommand>,
	pigeon_state_rx: Receiver<PigeonState>,
	state: PigeonState,
	components: PigeonComponents
}

struct PigeonComponents {
	relay: u8,
	status_running: u8,
	status_stopped: u8
}

impl Pigeon {
	fn new(
		components: PigeonComponents,
		power_cmd_rx: Receiver<PowerCommand>,
		pigeon_state_rx: Receiver<PigeonState>,
	) -> Result<Pigeon, GPIOError> {
		let mut gpio = try!(GPIO::new());

		// GPIO setup
		gpio.set_mode(components.relay, Mode::Output);
		gpio.set_mode(components.status_running, Mode::Output);
		gpio.set_mode(components.status_stopped, Mode::Output);

		gpio.write(components.relay, Level::Low);
		gpio.write(components.status_running, Level::Low);
		gpio.write(components.status_stopped, Level::High);


		Ok(Pigeon {
			gpio: gpio,
			power_cmd_rx: power_cmd_rx,
			pigeon_state_rx: pigeon_state_rx,
			state: PigeonState {
				operating_time: 20,
				release_time: 15,
			},
			components: components,
		})
	}

	fn start(&mut self) -> Box<Error> {
		loop {
			match self.power_cmd_rx.recv() {
				Ok(power_cmd) => if power_cmd.power {
					loop {
						match self.pigeon_state_rx.try_recv() {
							Ok(pigeon_state) => {
								self.state = pigeon_state;
							},
							// TODO: Differentiate between Empty & Disconnected
							Err(_) => {}
						}
						match self.power_cmd_rx.try_recv() {
							// TODO: More elegant solution
							Ok(power_cmd) => {
								if !power_cmd.power {
									break
								}
							}
							// TODO: Differentiate between Empty & Disconnected
							Err(_) => {}
						}
						self.gpio.write(self.components.relay, Level::High);
						sleep(Duration::from_millis(self.state.operating_time));
						self.gpio.write(self.components.relay, Level::Low);
						sleep(Duration::from_millis(self.state.release_time));
					}
				},
				Err(err) => return Box::new(err),
			}
		}
	}
}

fn main() {
	let (power_cmd_tx, power_cmd_rx) = channel();
	let (pigeon_state_tx, pigeon_state_rx) = channel();


	let mut pigeon = Pigeon::new(
		PigeonComponents {
			relay: RELAY,
			status_running: LED_GREEN,
			status_stopped: LED_RED,
		},
		power_cmd_rx,
		pigeon_state_rx,
	).unwrap();


	thread::spawn(move || { pigeon.start(); });

	let mut button = Button::new(BUTTON).unwrap();
	let power_cmd_tx_clone = power_cmd_tx.clone();


	thread::spawn(move || { 
		let mut power = false;

		button.poll(|| {
			let power_cmd_tx = &power_cmd_tx_clone;

			power = !power;

			println!("{:?}", power);


			// TODO: Use result
			power_cmd_tx.send(PowerCommand {power: power}).ok();

		})
	});	


	match TcpListener::bind("pigeon.local:1630") {
		Ok(listener) => for stream in listener.incoming() {
			let power_cmd_tx_clone = power_cmd_tx.clone();
			let pigeon_state_tx_clone = pigeon_state_tx.clone();

			thread::spawn(move || {
				let mut stream = BufReader::new(stream.unwrap());
				let mut raw_message: String;

				let power_cmd_tx = power_cmd_tx_clone;
				let pigeon_state_tx = pigeon_state_tx_clone;

				loop {
					raw_message = "".to_owned();

					stream.read_line(&mut raw_message).ok();


					match serde_json::from_str::<Message>(&raw_message) {
						Ok(message) => match message.cmd_type.as_ref() {
							"power" => {
								// TODO: Better solution
								let payload = message.payload.to_string();

								println!("{:?}", payload);

								// TODO: Input validation
								power_cmd_tx.send(
									serde_json::from_str::<PowerCommand>(&payload).unwrap(),
								).ok();
							}
							"pigeon_state" => {
								// TODO: Better solution
								let payload = message.payload.to_string();

								// TODO: Input validation
								pigeon_state_tx.send(
									serde_json::from_str::<PigeonState>(&payload).unwrap(),
								).ok();
							}
							_ => println!("Unrecognized command type: '{:?}'", message.cmd_type),
						},
						// TODO: Don't panic on invalid input
						Err(err) => panic!("{:?}", err),
					}
				}
			});
		},
		Err(err) => println!("{:?}", err),
	}
}
