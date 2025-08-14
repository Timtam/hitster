# node version
ARG NODE_VERSION=22

# pot provider version
ARG POT_PROVIDER_VERSION=1.2.1

# rust version
ARG RUST_VERSION=1.89.0

# s6-overlay version
ARG S6_OVERLAY_VERSION=3.2.1.0

# yt-dlp version
ARG YT_DLP_BUILD_VERSION=2025.08.11

FROM node:${NODE_VERSION} AS pot_provider_build_image

ARG POT_PROVIDER_VERSION

ENV POT_PROVIDER_VERSION ${POT_PROVIDER_VERSION}

RUN git clone --single-branch --branch ${POT_PROVIDER_VERSION} https://github.com/Brainicism/bgutil-ytdlp-pot-provider.git /pot-provider && \
    cd /pot-provider/server && \
    npm install && \
    npx tsc

FROM node:${NODE_VERSION} AS client_build_image

ARG HITSTER_BRANCH
ARG HITSTER_VERSION

ENV HITSTER_BRANCH ${HITSTER_BRANCH}
ENV HITSTER_VERSION ${HITSTER_VERSION}

WORKDIR /app

# build cache first

COPY ./client/package.json ./client/package-lock.json /app/

RUN npm install && rm /app/*.json 

# build everything else

COPY ./client/ /app/

RUN npm run build

FROM rust:${RUST_VERSION}-slim-bookworm AS server_build_image

# create a new empty shell project
RUN apt-get update && apt-get -y install libssl-dev pkg-config && \
    USER=root mkdir hitster

WORKDIR /hitster

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN USER=root cargo new --bin server && \
    USER=root cargo new --bin cli && \
    USER=root cargo new --lib core --name hitster_core

# copy over your manifests
COPY ./cli/Cargo.toml ./cli/Cargo.toml
COPY ./core/Cargo.toml ./core/Cargo.toml
COPY ./server/Cargo.toml ./server/Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release --no-default-features --features yt_dl && \
    cargo build --release -p hitster-cli && \
    rm server/src/*.rs && \
    rm core/src/*.rs && \
    rm cli/src/*.rs

# copy your source tree
COPY ./.sqlx ./.sqlx
COPY ./etc ./etc
COPY ./server/migrations ./server/migrations
COPY ./server/src ./server/src
COPY ./core/src ./core/src
COPY ./cli/src ./cli/src

# build for release
RUN rm ./target/release/deps/hitster* && \
    rm ./target/release/deps/libhitster* && \
    SQLX_OFFLINE=true cargo build --release --no-default-features --features yt_dl && \
    cargo build --release -p hitster-cli

# our final bases, platform-dependent

# x64
FROM debian:bookworm-slim AS build_amd64

ARG S6_OVERLAY_VERSION

ONBUILD ADD https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz /opt/ffmpeg.tar.xz
ONBUILD ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-x86_64.tar.xz /tmp/s6-overlay.tar.xz

# arm64
FROM debian:bookworm-slim AS build_arm64

ARG S6_OVERLAY_VERSION

ONBUILD ADD https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linuxarm64-gpl.tar.xz /opt/ffmpeg.tar.xz
ONBUILD ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-aarch64.tar.xz /tmp/s6-overlay.tar.xz

FROM build_${TARGETARCH}

ARG NODE_VERSION
ARG S6_OVERLAY_VERSION
ARG YT_DLP_BUILD_VERSION

WORKDIR /hitster

ENV CLIENT_DIRECTORY=/hitster/client
ENV DATABASE_URL=sqlite:///hitster.sqlite
ENV DOWNLOAD_DIRECTORY=/hits
ENV NODE_VERSION=${NODE_VERSION}
ENV PATH="$PATH:/opt/ffmpeg/bin/"

EXPOSE 8000

VOLUME [ "/hits", "/hitster.sqlite" ]

# install s6-overlay

ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz /tmp

# prepare the OS

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
    apt-get install -y curl && \
    curl -sL https://deb.nodesource.com/setup_${NODE_VERSION}.x | bash - && \
    apt-get -y install --no-install-recommends libssl-dev ca-certificates python3 python3-mutagen python3-pip xz-utils nodejs && \
    pip3 install --no-cache-dir --break-system-packages ffmpeg-normalize && \
    mkdir /opt/ffmpeg && \
    tar xf /opt/ffmpeg.tar.xz -C /opt/ffmpeg/ --strip-components 1 && \
    tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay.tar.xz && \
    apt-get purge -y --auto-remove python3-pip xz-utils curl && \
    apt-get clean && \
    rm /opt/ffmpeg.tar.xz && \
    rm /tmp/s6-overlay-noarch.tar.xz && \
    rm /tmp/s6-overlay.tar.xz && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir /.cache && \
    chmod 777 /.cache && \
    echo "--ffmpeg-location /opt/ffmpeg/bin/" > /etc/yt-dlp.conf

# yt-dlp

ADD --chmod=777 https://github.com/yt-dlp/yt-dlp/releases/download/${YT_DLP_BUILD_VERSION}/yt-dlp /usr/local/bin/yt-dlp

# copy the build artifacts from the build stage
COPY --from=server_build_image /hitster/target/release/hitster-server /hitster/server
COPY --from=server_build_image /hitster/target/release/hitster-cli /hitster/cli
COPY --from=client_build_image /app/dist /hitster/client
COPY --from=pot_provider_build_image /pot-provider/server/build /pot-provider/build
COPY --from=pot_provider_build_image /pot-provider/server/node_modules /pot-provider/node_modules
COPY --from=pot_provider_build_image /pot-provider/plugin /etc/yt-dlp/plugins/bgutil-ytdlp-pot-provider

# setup s6-overlay
COPY ./docker/s6-rc.d /etc/s6-overlay/s6-rc.d

# set the startup command to run your binary
ENTRYPOINT [ "/init" ]
