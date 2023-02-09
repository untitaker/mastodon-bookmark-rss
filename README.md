# mastodon-bookmark-rss

A small web service to let you connect your mastodon bookmarks to your RSS reader.

Use the instance at [woodland.cafe](https://bookmark-rss.services.woodland.cafe) or explore the options for self-hosting below.

## Operating it yourself

You can use the [pre-built docker
images](https://github.com/untitaker/mastodon-bookmark-rss/pkgs/container/mastodon-bookmark-rss)
behind some sort of reverse proxy.

Note that if this service is exposed to the internet as-is, anybody will be
able to use it. You will have to figure out by yourself how to lock it down in
a way that works with your RSS reader.

If you are running an open
instance to be used by anybody, please make sure that your reverse proxy sends
the corresponding proxy headers as documented [here for IP
addresses](https://docs.rs/tower_governor/0.0.4/tower_governor/key_extractor/struct.SmartIpKeyExtractor.html)
and [here for
hostnames](https://docs.rs/axum/0.6.4/axum/extract/struct.Host.html). Those
are used to enforce (currently hardcoded) per-IP rate limits, and to send the
app's own hostname as part of the user-agent that Mastodon admins can see.

## Running it locally

1. Clone this repository
2. Have Rust, Node and Yarn installed. I recommend [rustup](https://rustup.rs/)
   for Rust and [Volta](https://volta.sh/) for installing both Node and Yarn.
3. Install [entr](http://eradman.com/entrproject/)
4. Run `yarn install && yarn dev` to get a devserver at `localhost:3000`

To get a standalone release binary, run `yarn install && yarn build && cargo build --release`.

## License

MIT, see `LICENSE`.
