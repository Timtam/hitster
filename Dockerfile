FROM node:20 AS client_build_image

WORKDIR /app

COPY ./client /app
COPY ./server/Cargo.toml /app/Cargo.toml

RUN npm install && npm run build

FROM clux/muslrust:1.79.0-stable AS server_build_image

# create a new empty shell project
RUN echo $pwd && \
    USER=root cargo new --bin /hitster

WORKDIR /hitster

# copy over your manifests
COPY ./server/Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release && \
    rm ./src/*.rs

# copy your source tree
COPY ./server/migrations ./migrations
COPY ./server/src ./src
COPY ./server/build.rs ./build.rs
COPY ./server/etc ./etc

# build for release
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/hitster* && \
    cargo build --release

FROM alpine:3.20

# yt-dlp version
ARG YT_DLP_BUILD_VERSION=2024.07.09

WORKDIR /hitster

ENV CLIENT_DIRECTORY=/hitster/client
ENV PATH="$PATH:/opt/ffmpeg/bin/"
ENV USE_YT_DLP=true
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

# prepare the OS

RUN set -x && \
    apk update && \
    apk upgrade -a && \
    apk add --no-cache ca-certificates ffmpeg python3 py3-mutagen && \
    mkdir /.cache && \
    chmod 777 /.cache

# copy the build artifact from the build stage
COPY --from=server_build_image /hitster/target/x86_64-unknown-linux-musl/release/hitster-server /hitster/server/hitster
COPY --from=client_build_image /app/dist /hitster/client

# yt-dlp

ADD --chmod=777 https://github.com/yt-dlp/yt-dlp/releases/download/${YT_DLP_BUILD_VERSION}/yt-dlp /usr/local/bin/yt-dlp

# set the startup command to run your binary
CMD ["/hitster/server/hitster"]
