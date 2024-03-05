FROM rust:1.76-slim-bookworm as build

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

# our final base
FROM debian:bookworm-slim

# prepare the OS

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
    apt-get -y install --no-install-recommends libssl-dev ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# copy the build artifact from the build stage
COPY --from=build /hitster/target/release/hitster-server ./hitster

# set the startup command to run your binary
CMD ["./hitster"]