# redispeech
## What is this?
I needed a quick way to process networked audio and pass it into a speech recognition engine. This will serve as a backbone for many future projects :).
## Prerequsites
### Installation
* python3
	- Modules: redis, wave, numpy
*  [docker](https://docs.docker.com/install/)
## Test
You will need to download some model data and place them in the models directory. I used the models provided by the deepspeech distribution: https://github.com/mozilla/DeepSpeech/releases/download/v0.5.1/deepspeech-0.5.1-models.tar.gz

Start up the project:
```bash
docker-compose up
```
In a separate terminal:
```bash
docker exec redis redis-cli monitor
```
In another terminal:
```bash
cd test
python test/test.py
```
You should see some activity in the terminal where you run redis-cli.
