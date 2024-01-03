FROM docker.io/rust:1.74 as builder
RUN apt-get update && apt-get install -y locales libc6 libssl-dev && rm -rf /var/lib/apt/lists/* \
    && localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8
WORKDIR /usr/src/myapp
COPY Cargo.toml Cargo.lock .
COPY rust-toolchain-deploy.toml rust-toolchain.toml
COPY src src
ENV VOTERS_VERDICT_ENVIRONMENT=prod
ARG TARGET
RUN cargo build --features='templates' --release --bin voters-verdict-machine --target ${TARGET}-unknown-linux-gnu

FROM docker.io/node:latest as builderNode
WORKDIR /usr/src
COPY static static
COPY package.json package-lock.json webpack.config.js /usr/src/
RUN npm install && npx webpack-cli build


FROM docker.io/debian:bookworm-slim
RUN apt-get update && apt-get install -y locales libc6 libssl-dev && rm -rf /var/lib/apt/lists/* \
    && localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8
COPY --from=builder /usr/src/myapp/target/x86_64-unknown-linux-gnu/release/voters-verdict-machine /usr/local/bin/voters-verdict-machine
COPY --from=builderNode /usr/src/minified /usr/src/myapp/static

WORKDIR /usr/src/myapp
RUN mkdir -p bucket/candidates bucket/ballots bucket/votings bucket/criteria
RUN mkdir -p /tmp/candidates /tmp/votings /tmp/criteria /tmp/ballots

COPY assets/styles /usr/src/myapp/assets/styles
COPY templates templates
COPY Rocket.toml .
ARG DATE_DEPLOY
ENV VOTERS_VERDICT_DATE_DEPLOY=${DATE_DEPLOY}
ENV RUST_LOG=info

# Use docker-compose to set all these values
# ENV VOTERS_VERDICT_ADMIN_TOKEN=
# ENV VOTERS_VERDICT_MAINTAINER_TOKEN=

# ENV VOTERS_VERDICT_DEFAULT_VOTE_RUNNING=

# Overwrite or change them to default prepared votings
# Or mount it
ENV VOTERS_VERDICT_FILE_DIR=/usr/src/myapp/bucket/
ENV VOTERS_VERDICT_ASSET_DIR=/usr/src/myapp/static/

# If you want to connect to a readonly remote location.
# DB is not available!
# ENV VOTERS_VERDICT_SELF_CERT=
# ENV VOTERS_VERDICT_STORAGE_MODE=
# ENV VOTERS_VERDICT_REMOTE_STORAGE=
# ENV VOTERS_VERDICT_REMOTE_CREDENTIALS=
# ENV VOTERS_VERDICT_REMOTE_AUTH=

EXPOSE 9999
CMD ["voters-verdict-machine"]
