# mastodon-bookmark-rss

A small web service to let you connect your mastodon bookmarks to your RSS reader.

Use the instance at [woodland.cafe](https://bookmark-rss.services.woodland.cafe) or explore the options for self-hosting below.

## Operating it yourself

Please see [docs/deploy.md](docs/deploy.md).

## Running it locally

1. Clone this repository
2. Have Rust, Node and Yarn installed. I recommend [rustup](https://rustup.rs/)
   for Rust and [Volta](https://volta.sh/) for installing both Node and Yarn.
3. Install [entr](http://eradman.com/entrproject/)
4. Run `yarn install && yarn dev` to get a devserver at `localhost:3000`

To get a standalone release binary, run `yarn install && yarn build && cargo build --release`.

## License

MIT, see `LICENSE`.
