from deepspeech import Model
import numpy as np

import sys, json, os, time

import logging
logging.basicConfig(format='%(asctime)s', level=logging.DEBUG)
logger = logging.getLogger(__name__)

import redis
redis_instance = redis.Redis(host=os.environ['REDIS_HOST'], port=os.environ['REDIS_PORT'], db=0)
audio_pubsub = redis_instance.pubsub()

def setup_model(args):
		
	# These constants control the beam search decoder

	# Beam width used in the CTC decoder when building candidate transcriptions
	BEAM_WIDTH = 500

	# The alpha hyperparameter of the CTC decoder. Language Model weight
	LM_ALPHA = 0.75

	# The beta hyperparameter of the CTC decoder. Word insertion bonus.
	LM_BETA = 1.85


	# These constants are tied to the shape of the graph used (changing them changes
	# the geometry of the first layer), so make sure you use the same constants that
	# were used during training

	# Number of MFCC features to use
	N_FEATURES = 26

	# Size of the context window used for producing timesteps in the input vector
	N_CONTEXT = 9
	ds = Model(args['model'], N_FEATURES, N_CONTEXT, args['alphabet'], BEAM_WIDTH)
	ds.enableDecoderWithLM(args['alphabet'], args['lm'], args['trie'], LM_ALPHA, LM_BETA)
	return ds

def make_handler_interpret_audio(ds):
	def handler(audio):
		audio_length = len(audio['data'])
		if audio_length % 16 == 0:
			metadata = ds.sttWithMetadata(np.frombuffer(audio['data'], np.int16), 16000)
			reply = ''.join(item.character for item in metadata.items)
			reply_channel = 'interpreted-{}'.format(audio['channel'].decode("utf-8") )
			redis_instance.publish(reply_channel, reply)
		else:
			print('Audio buffer had {} extra entries'.format(audio_length % 16000))
	return handler

def main():
	config = sys.argv[1]
	with open(config) as json_file:  
		args = json.load(json_file)
	ds = setup_model(args)
	audio_pubsub.psubscribe(**{'audio-*': make_handler_interpret_audio(ds)})
	for message in audio_pubsub.listen():
		print(message, file=sys.stdout)

if __name__ == '__main__':
	main()