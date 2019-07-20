// extern crate audrey;
extern crate deepspeech;
extern crate serde;
extern crate serde_json;

use std::env::args;
use std::fs;
use std::path::Path;
use std::{thread, time};

use byteorder::{ByteOrder, LittleEndian};
use deepspeech::Model;

use std::sync::mpsc;
use std::sync::mpsc::SyncSender;

use std::env;

use std::collections::HashMap;

use lapin::{
    message::Delivery, options::*, types::FieldTable, BasicProperties, Channel, Connection,
    ConnectionProperties, ConsumerSubscriber,
};
use lapin_async as lapin;

use std::fmt;

// These constants are taken from the C++ sources of the client.
const N_CEP: u16 = 26;
const N_CONTEXT: u16 = 9;
const BEAM_WIDTH: u16 = 500;
const SAMPLE_RATE: u32 = 16_000;

const LM_WEIGHT: f32 = 0.75;
const VALID_WORD_COUNT_WEIGHT: f32 = 1.85;

const HALF_SECOND: time::Duration = time::Duration::from_millis(500);

struct Subscriber {
    channel: Channel,
    sync_sender: SyncSender<Vec<u8>>,
}

impl fmt::Debug for Subscriber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.channel)
    }
}

impl ConsumerSubscriber for Subscriber {
    fn new_delivery(&self, delivery: Delivery) {
        self.channel
            .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
            .as_error()
            .expect("basic_ack");
        if let Ok(_) = self.sync_sender.send(delivery.data) {
            println!("Data sent");
        }
    }
    fn drop_prefetched_messages(&self) {}
    fn cancel(&self) {}
}

pub fn send_message(publish_channel: &Channel, queue_name: &str, payload: &[u8]) {
    publish_channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .wait()
        .expect("queue_declare");
    publish_channel
        .basic_publish(
            "",
            queue_name,
            BasicPublishOptions::default(),
            payload.to_vec(),
            BasicProperties::default(),
        )
        .wait()
        .expect("basic_publish");
    println!("Payload sent to {}", queue_name);
}

pub fn attach_consumer(
    queue_name: &str,
    consumer_name: &'static str,
    conn: Connection,
    subcribe_channels: &mut HashMap<&'static str, Channel>,
    sync_sender: &SyncSender<Vec<u8>>,
) {
    let channel: &Channel = subcribe_channels
        .entry(consumer_name)
        .or_insert(conn.create_channel().wait().expect("create_channel"));

    let queue = channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .wait()
        .expect("queue_declare");
    channel
        .basic_consume(
            &queue,
            consumer_name,
            BasicConsumeOptions::default(),
            FieldTable::default(),
            Box::new(Subscriber {
                channel: channel.clone(),
                sync_sender: sync_sender.clone(),
            }),
        )
        .wait()
        .expect("basic_consume");
    println!("Consumer attached to {}", queue_name);
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
    let config: serde_json::Value =
        serde_json::from_str(&contents).expect("JSON was not well-formatted");

    let broker_host: &str = &env::var("BROKER_HOST").unwrap();
    let broker_port: &str = &env::var("BROKER_PORT").unwrap();

    let model_file: &Path = Path::new(config["model"].as_str().unwrap());
    let alphabet_file: &Path = &Path::new(config["alphabet"].as_str().unwrap());
    let binary_file: &Path = &Path::new(config["binary"].as_str().unwrap());
    let trie_file: &Path = &Path::new(config["trie"].as_str().unwrap());

    let model_result =
        Model::load_from_files(model_file, N_CEP, N_CONTEXT, alphabet_file, BEAM_WIDTH);
    match model_result {
        Ok(_) => println!("Model loaded"),
        Err(err) => panic!("{:?}", err),
    };
    let mut model: Model = model_result.unwrap();
    model.enable_decoder_with_lm(
        alphabet_file,
        binary_file,
        trie_file,
        LM_WEIGHT,
        VALID_WORD_COUNT_WEIGHT,
    );

    let conn: Connection = Connection::connect(
        &format!("amqp://{}:{}/%2f", broker_host, broker_port),
        ConnectionProperties::default(),
    )
    .wait()
    .expect("connection error");
    let publish_channel: Channel = conn.create_channel().wait().expect("create_channel");
    let mut subcribe_channels: HashMap<&str, Channel> = HashMap::new();

    let (sync_sender, receiver) = mpsc::sync_channel(1);
    const QUEUE_NAME: &str = "audio";
    const CONSUMER_NAME: &str = "interpret";
    thread::spawn(move || {
        attach_consumer(
            QUEUE_NAME,
            CONSUMER_NAME,
            conn,
            &mut subcribe_channels,
            &sync_sender,
        )
    });
    loop {
        if let Ok(audio_buf) = receiver.recv() {
            let mut converted: Vec<i16> = vec![0; audio_buf.len() / 2];
            LittleEndian::read_i16_into(&audio_buf, &mut converted);

            let result: String = model.speech_to_text(&converted, SAMPLE_RATE).unwrap();
            send_message(&publish_channel, "interpreted", result.as_bytes());

            println!("{}", result);
        }
        thread::sleep(HALF_SECOND);
    }
}
