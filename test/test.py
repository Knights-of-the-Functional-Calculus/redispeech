import redis, wave
import numpy as np

redis_instance = redis.Redis(host='0.0.0.0', port=6379, db=0)

fin = wave.open('../audio/2830-3980-0043.wav', 'rb')
audio = np.frombuffer(fin.readframes(fin.getnframes()), np.int16)
print(redis_instance.publish('audio-test', audio.tobytes()))
print(redis_instance.publish('audio-test', 'test done'))
