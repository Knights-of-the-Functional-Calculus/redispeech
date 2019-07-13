extern crate audrey;
extern crate deepspeech;
extern crate serde;
extern crate serde_json;
extern crate scrap_merchant;

use std::env::args;
use std::fs;
use std::path::Path;

// use audrey::read::Reader;
// use audrey::sample::signal::{from_iter, Signal};
use deepspeech::Model;
use scrap_merchant::rabbitmq::{Broker, Caller};

use std::env;
// These constants are taken from the C++ sources of the client.

const N_CEP: u16 = 26;
const N_CONTEXT: u16 = 9;
const BEAM_WIDTH: u16 = 500;

const LM_WEIGHT: f32 = 0.75;
const VALID_WORD_COUNT_WEIGHT: f32 = 1.85;

// The model has been trained on this specific
// sample rate.
const SAMPLE_RATE: u32 = 16_000;

const BROKER_HOST: &str = &env::var("BROKER_HOST").unwrap();
const BROKER_PORT: &str = &env::var("BROKER_PORT").unwrap();

 fn handle_interpret_audio(audio_buf: &[i16]) {
        // Run the speech to text algorithm
        let result = self.model.speech_to_text(&audio_buf, SAMPLE_RATE).unwrap();
        self.broker.send_message("interpreted", result.as_bytes());
        println!("{}", result);
	}

/*
TODO list:
* better resampling (right now it seems that recognition is impaired compared to manual resampling)...
  maybe use sinc?
* channel cropping
* use clap or something to parse the command line arguments
*/
fn main() {
    let contents = fs::read_to_string(&args().nth(1).unwrap()).unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&contents).expect("JSON was not well-formatted");
    let mut m : Model = Model::load_from_files(
        &Path::new(json["model"].as_str().unwrap()),
        N_CEP,
        N_CONTEXT,
        &Path::new(json["alphabet"].as_str().unwrap()),
        BEAM_WIDTH,
    )
    .unwrap();
    m.enable_decoder_with_lm(
        &Path::new(json["alphabet"].as_str().unwrap()), &Path::new(json["binary"].as_str().unwrap()),
        &Path::new(json["trie"].as_str().unwrap()),
        LM_WEIGHT,
        VALID_WORD_COUNT_WEIGHT,
    );

    let broker: Broker;
    broker.init(&format!("{}:{}", BROKER_HOST, BROKER_PORT));

    broker.attach_consumer("audio", "interpret",  &handle_interpret_audio);
}
