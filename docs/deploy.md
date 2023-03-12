# Operating mastodon-bookmark-rss yourself

Note that if this service is exposed to the internet as-is, anybody will be
able to use it. You will have to figure out by yourself how to lock it down in
a way that works with your RSS reader.

The app is pretty much a thin proxy for `GET /api/v1/bookmarks` to arbitrary
hosts. You are responsible for any abuse your hosting of this app enables. The
app already hardcodes a few default rate limits that should suffice, but as
always, this software comes with no strings attached.

## Fly.io

[Fly.io](https://fly.io) can be used to host this app for free.

1. Sign up on Fly.io and install their CLI tool `flyctl`
2. Clone this repo, and edit `fly.toml` as per instructions in
   comments
3. Run `flyctl launch` in it

You will get a subdomain at `https://*.fly.dev` with SSL.

Fly.io mostly prices by egress bandwidth, but for personal use it is highly
unlikely that you will exceed the free contingent.

Alternatives to Fly.io are [Render](https://render.com) and
[Railway](https://railway.app). They are pretty much equivalent in pricing and
ease of use. I've managed to deploy this app with all of them.

## Other (VPS)

Besides the options above, you can host this app on anything where docker can run.

[pre-built docker
images](https://github.com/untitaker/mastodon-bookmark-rss/pkgs/container/mastodon-bookmark-rss) are available.

If you don't want to use docker, you can run `cargo build --release` to get a binary that only depends on `libc`.

Please make sure that your reverse proxy sends
the corresponding proxy headers as documented [here for IP
addresses](https://docs.rs/tower_governor/0.0.4/tower_governor/key_extractor/struct.SmartIpKeyExtractor.html)
and [here for
hostnames](https://docs.rs/axum/0.6.4/axum/extract/struct.Host.html). Those
are used to enforce (currently hardcoded) per-IP rate limits, and to send the
app's own hostname as part of the user-agent that Mastodon admins can see.
