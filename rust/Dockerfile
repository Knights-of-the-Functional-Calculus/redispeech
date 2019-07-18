FROM rust:latest

RUN apt-get update -y && \
			apt-get install -y clang

WORKDIR /deepspeech
COPY . .


ADD https://github.com/mozilla/DeepSpeech/releases/download/v0.5.1/native_client.amd64.cpu.linux.tar.xz deepspeech-nacl.tar.xz
RUN tar xvf deepspeech-nacl.tar.xz && cp libdeepspeech.so /usr/lib

RUN cargo install --path .

COPY config.json config.json

ENV MODEL_PACKAGE https://github.com/mozilla/DeepSpeech/releases/download/v0.5.1/deepspeech-0.5.1-models.tar.gz

ADD ${MODEL_PACKAGE} models.tar.gz
RUN mkdir ../models
RUN tar xvfz models.tar.gz ../models

CMD ["rust"]