FROM node:20 AS client_build_image

ARG HITSTER_VERSION

WORKDIR /app

# build cache first

COPY ./client/package.json ./client/package-lock.json /app/

RUN npm install && rm /app/*.json 

# build everything else

COPY ./client/ /app/

RUN npm run build

FROM rust:1.82-slim-bookworm AS server_build_image

# create a new empty shell project
RUN apt-get update && apt-get -y install libssl-dev pkg-config && \
    USER=root cargo new --bin hitster

WORKDIR /hitster

# copy over your manifests
COPY ./server/Cargo.lock ./Cargo.lock
COPY ./server/Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release && \
    rm src/*.rs

# copy your source tree
COPY ./server/migrations ./migrations
COPY ./server/src ./src
COPY ./server/build.rs ./build.rs
COPY ./server/etc ./etc

# build for release
RUN rm ./target/release/deps/hitster* && \
    cargo build --release

# our final bases, platform-dependent

# x64
FROM debian:bookworm-slim AS build_amd64

ONBUILD ADD https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz /opt/ffmpeg.tar.xz

# arm64
FROM debian:bookworm-slim AS build_arm64

ONBUILD ADD https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linuxarm64-gpl.tar.xz /opt/ffmpeg.tar.xz

FROM build_${TARGETARCH}

# yt-dlp version
ARG YT_DLP_BUILD_VERSION=2024.10.22

WORKDIR /hitster

ENV CLIENT_DIRECTORY=/hitster/client
ENV DATABASE_URL=sqlite:///hitster.sqlite
ENV DOWNLOAD_DIRECTORY=/hits
ENV PATH="$PATH:/opt/ffmpeg/bin/"
ENV USE_YT_DLP=true

EXPOSE 8000

VOLUME [ "/hits", "/hitster.sqlite" ]

# prepare the OS

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
    apt-get -y install --no-install-recommends libssl-dev ca-certificates python3 python3-mutagen python3-pip xz-utils && \
    pip3 install --no-cache-dir --break-system-packages ffmpeg-normalize && \
    mkdir /opt/ffmpeg && \
    tar xf /opt/ffmpeg.tar.xz -C /opt/ffmpeg/ --strip-components 1 && \
    apt-get purge -y --auto-remove python3-pip xz-utils && \
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
