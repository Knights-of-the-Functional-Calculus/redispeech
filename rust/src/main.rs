extern crate audrey;
extern crate deepspeech;
extern crate serde;
extern crate serde_json;

use std::env::args;
use std::fs;
use std::path::Path;

// use audrey::read::Reader;
// use audrey::sample::signal::{from_iter, Signal};
// use byteorder::{ByteOrder, LittleEndian};
use deepspeech::Model;

use queues::{IsQueue, Queue};
use std::env;
// These constants are taken from the C++ sources of the client.

use std::collections::HashMap;

use lapin::{
    message::Delivery, options::*, types::FieldTable, BasicProperties, Channel, Connection,
    ConnectionProperties, ConsumerSubscriber,
};
use lapin_async as lapin;

use std::fmt;

const N_CEP: u16 = 26;
const N_CONTEXT: u16 = 9;
const BEAM_WIDTH: u16 = 500;

const LM_WEIGHT: f32 = 0.75;
const VALID_WORD_COUNT_WEIGHT: f32 = 1.85;

struct Subscriber {
    channel: Channel,
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
    print!("Payload sent to {}", queue_name);
}

pub fn attach_consumer(
    conn: Connection,
    mut subcribe_channels: HashMap<&str, Channel>,
    message_passer: &mut Queue<Vec<u8>>,
) {
    let queue_name = "audio";
    let consumer_name = "interpret";
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
            }),
        )
        .wait()
        .expect("basic_consume");
    println!("Consumer attached to {}", queue_name);
    // get the next message
    if let Ok(message) = channel
        .basic_get(queue_name, BasicGetOptions::default())
        .wait()
    {
        let data = message.unwrap().delivery.data;
        if let Ok(_) = message_passer.add(data) {
            println!("Data store in local queue");
        }
    }
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

    // let modelManager: ModelManager = ModelManager::new(json);
    // let mut callback: Box<FnMut(Vec<u8>)> = Box::new(|audio_buf: Vec<u8>| {
    //     let mut result;
    //     let mut converted: &mut [i16];
    //     LittleEndian::read_i16_into(&audio_buf, &mut converted);
    //     unsafe {
    //         result = modelManager
    //             .model
    //             .speech_to_text(converted, SAMPLE_RATE)
    //             .unwrap();
    //         modelManager
    //             .broker
    //             .send_message("interpreted", result.as_bytes());
    //     }

    //     println!("{}", result);
    // });

    // The model has been trained on this specific
    // sample rate.
    // let SAMPLE_RATE: u32 = 16_000;

    let BROKER_HOST: &str = &env::var("BROKER_HOST").unwrap();
    let BROKER_PORT: &str = &env::var("BROKER_PORT").unwrap();

    let mut m: Model = Model::load_from_files(
        &Path::new(config["model"].as_str().unwrap()),
        N_CEP,
        N_CONTEXT,
        &Path::new(config["alphabet"].as_str().unwrap()),
        BEAM_WIDTH,
    )
    .unwrap();
    m.enable_decoder_with_lm(
        &Path::new(config["alphabet"].as_str().unwrap()),
        &Path::new(config["binary"].as_str().unwrap()),
        &Path::new(config["trie"].as_str().unwrap()),
        LM_WEIGHT,
        VALID_WORD_COUNT_WEIGHT,
    );

    let mut conn: Connection = Connection::connect(
        &format!("{}:{}", BROKER_HOST, BROKER_PORT),
        ConnectionProperties::default(),
    )
    .wait()
    .expect("connection error");
    let mut publish_channel: Channel = conn.create_channel().wait().expect("create_channel");
    let mut subcribe_channels: HashMap<&str, Channel> = HashMap::new();
    // let modelManager: ModelManager = ModelManager {
    //     broker: &broker,
    //     model: &m,
    // };

    let mut queue: Queue<Vec<u8>> = Queue::new();
    attach_consumer(conn, subcribe_channels, &mut queue);
}
