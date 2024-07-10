FROM node:20 AS client_build_image

WORKDIR /app

COPY ./client /app
COPY ./server/Cargo.toml /app/Cargo.toml

RUN npm install && npm run build

FROM rust:1.79-slim-bookworm AS server_build_image

# create a new empty shell project
RUN apt-get update && apt-get -y install libssl-dev pkg-config && \
    USER=root cargo new --bin hitster
WORKDIR /hitster

# copy over your manifests
COPY ./server/Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./server/migrations ./migrations
COPY ./server/src ./src
COPY ./server/build.rs ./build.rs
COPY ./server/etc ./etc

# build for release
RUN rm ./target/release/deps/hitster*
RUN cargo build --release

# our final bases, platform-dependent

# x64
FROM debian:bookworm-slim AS build_amd64

ONBUILD ADD https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz /opt/ffmpeg.tar.xz

# arm64
FROM debian:bookworm-slim AS build_arm64

ONBUILD ADD https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linuxarm64-gpl.tar.xz /opt/ffmpeg.tar.xz

FROM build_${TARGETARCH}

# yt-dlp version
ARG YT_DLP_BUILD_VERSION=2024.07.09

WORKDIR /hitster

ENV CLIENT_DIRECTORY=/hitster/client
ENV PATH="$PATH:/opt/ffmpeg/bin/"

# prepare the OS

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
    apt-get -y install --no-install-recommends libssl-dev ca-certificates python3 python3-mutagen xz-utils && \
    mkdir /opt/ffmpeg && \
    tar xf /opt/ffmpeg.tar.xz -C /opt/ffmpeg/ --strip-components 1 && \
    apt-get purge -y --auto-remove xz-utils && \
    apt-get clean && \
    rm /opt/ffmpeg.tar.xz && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir /.cache && \
    chmod 777 /.cache && \
    echo "--ffmpeg-location /opt/ffmpeg/bin/" > /etc/yt-dlp.conf

# copy the build artifact from the build stage
COPY --from=server_build_image /hitster/target/release/hitster-server /hitster/server/hitster
COPY --from=client_build_image /app/dist /hitster/client

# yt-dlp

ADD --chmod=777 https://github.com/yt-dlp/yt-dlp/releases/download/${YT_DLP_BUILD_VERSION}/yt-dlp /usr/local/bin/yt-dlp

# set the startup command to run your binary
CMD ["/hitster/server/hitster"]
