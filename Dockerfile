FROM rust:1.66-alpine3.16 as builder

RUN apk add --no-cache libc-dev

RUN USER=root cargo new --bin /app
WORKDIR /app

# Just copy the Cargo.toml files and trigger
# a build so that we compile our dependencies only.
# This way we avoid layer cache invalidation
# if our dependencies haven't changed,
# resulting in faster builds.

COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build --release && rm -rf src/
RUN strip target/release/mastodon-bookmark-rss

# Copy the source code and run the build again.
# This should only compile the app itself as the
# dependencies were already built above.
COPY . ./
RUN rm ./target/release/deps/mastodon_bookmark_rss* && cargo build --release

# Our production image starts here, which uses
# the files from the builder image above.
FROM alpine:3.16

COPY --from=builder /app/target/release/mastodon-bookmark-rss /usr/local/bin/mastodon-bookmark-rss

RUN apk add --no-cache tini

RUN addgroup -S app && adduser -S app -G app
USER app

ENTRYPOINT ["/sbin/tini", "--", "mastodon-bookmark-rss"]
EXPOSE 3000
