FROM python:3

WORKDIR /usr/src/app

COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt
COPY src src
COPY ../config.json config.json

ENV MODEL_PACKAGE https://github.com/mozilla/DeepSpeech/releases/download/v0.5.1/deepspeech-0.5.1-models.tar.gz

ADD ${MODEL_PACKAGE} models.tar.gz
RUN tar xvfz models.tar.gz models

CMD [ "python", "./src/main.py", "config.json" ]