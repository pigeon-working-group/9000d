#[macro_use]
extern crate serde_derive;

#[macro_use] 
extern crate validator_derive;
extern crate validator;

extern crate serde_json;

extern crate rppal;

extern crate bufstream;

use rppal::gpio::{Level, Mode, GPIO};
use rppal::gpio::Error as GPIOError;

use validator::{Validate};

use bufstream::BufStream;

use std::thread::sleep;
use std::time::Duration;
use std::io::BufRead;
use std::net::{TcpListener};
use std::thread::spawn;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::error::Error;
use std::net::Shutdown;

const RELAY: u8 = 18;


fn generate_pigeon_state(ratio_state: PigeonStateRatio) -> PigeonState {
	let release_ratio = 1.0 - (ratio_state.operating_ratio);

	PigeonState {
		power: ratio_state.power,
		operating_time: 
			(ratio_state.operating_ratio * ratio_state.cycle_time) as u64,

		release_time: (release_ratio * ratio_state.cycle_time) as u64
	}
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Copy)]
struct PigeonStateRatio {
	power: bool,
	#[validate(range(min = "25", max = "1000"))]
	cycle_time: f32,
	#[validate(range(min = "0.0", max = "1.0"))]
	operating_ratio: f32,
}

#[derive(Debug)]
struct PigeonState {
	power: bool,
	operating_time: u64,
	release_time: u64,
}


struct Pigeon {
	gpio: GPIO,
	state_rx: Receiver<PigeonState>,
	components: PigeonComponents
}

struct PigeonComponents {
	relay: u8
}

impl Pigeon {
	fn new(
		components: PigeonComponents,
		state_rx: Receiver<PigeonState>,
	) -> Result<Pigeon, GPIOError> {
		let mut gpio = try!(GPIO::new());

		// GPIO setup
		gpio.set_mode(components.relay, Mode::Output);
		gpio.write(components.relay, Level::Low);



		Ok(Pigeon {
			gpio: gpio,
			state_rx: state_rx,
			components: components,
		})
	}

	fn start(&mut self) -> Box<Error> {
		let mut pigeon_state: PigeonState;

		loop {
			match self.state_rx.recv() {
				Ok(state) =>  {
					pigeon_state = state;

					println!("{:?}", pigeon_state);

					if pigeon_state.power {
						loop {
							match self.state_rx.try_recv() {
								Ok(state) => {
									pigeon_state = state;

									println!("{:?}", pigeon_state);

									if !pigeon_state.power {
										break;
									}
								},
								Err(err) => {
									if err != TryRecvError::Empty {
										return Box::new(err);
									}
								}
							}

							self.gpio.write(self.components.relay, Level::High);
							sleep(Duration::from_millis(pigeon_state.operating_time));
							self.gpio.write(self.components.relay, Level::Low);
							sleep(Duration::from_millis(pigeon_state.release_time));
						}
					}
				},
				Err(err) => return Box::new(err)
			}
		}
	}
}


fn main() {
	let (state_tx, state_rx) = channel();

	let mut pigeon = Pigeon::new(
		PigeonComponents {
			relay: RELAY
		},
		state_rx,
	).unwrap();

	
	spawn(move || { pigeon.start(); });

	match TcpListener::bind("0.0.0.0:1630") {
		Ok(listener) => for stream in listener.incoming() {
			let state_tx_clone = state_tx.clone();

			let mut buf = BufStream::new(stream.unwrap());


			spawn(move || {
				let mut raw_message: String;

				let state_tx = state_tx_clone;

				loop {
					//let buffers = buffers_clone.clone().lock().unwrap();

					raw_message = "".to_owned();

					if buf.read_line(&mut raw_message).is_ok() {
						match serde_json::from_str::<PigeonStateRatio>(&raw_message) {
							Ok(state) => {
								if let Err(err) = state_tx.send(generate_pigeon_state(state)) {
									// If messaging dies, there is something seriously wrong
									panic!("Could not send message '{:?}'", err)
								}
							},
							Err(err) =>  {
								if err.is_eof() {

									buf.get_ref().shutdown(Shutdown::Both).ok();

									return;
								} else {
									println!("{:?}", err);
								}
							}
						}						
					} else {
						// Better error handling
						return;
					}
				}
			});
		},
		Err(err) => panic!("{:?}", err)
	}
}
