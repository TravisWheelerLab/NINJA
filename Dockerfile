FROM alpine:3.10

LABEL description="A build image for NINJA"
LABEL url="https://github.com/TravisWheelerLab/NINJA"
LABEL version="1.0.0"

# Dependencies
RUN apk add --no-cache \
    clang \
    g++ \
    git \
    make

# Location of our code
VOLUME /code
WORKDIR /code

ENTRYPOINT exec make

