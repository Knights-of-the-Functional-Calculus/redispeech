import pika
import wave
import numpy as np
import logging
logging.getLogger(__name__).setLevel(logging.INFO)

fin = wave.open('../../audio/2830-3980-0043.wav', 'rb')
audio = np.frombuffer(fin.readframes(fin.getnframes()), np.int16)

parameters = pika.ConnectionParameters('localhost')
connection = pika.BlockingConnection(parameters)
channel = connection.channel()
channel.queue_declare(queue='audio')
channel.basic_publish(exchange='', routing_key='audio',
                      body=audio.tobytes())

while True:
	pass

connection.close()