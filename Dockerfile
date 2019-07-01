FROM python:3

WORKDIR /usr/src/app

COPY models models
COPY audio audio

COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt
COPY src src
COPY test test
COPY config.json config.json

CMD [ "python", "./src/main.py", "config.json" ]