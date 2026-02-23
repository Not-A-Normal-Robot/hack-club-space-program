# Hack Club Space Program

A space simulator prototype made for
[Hack Club Flavortown](https://flavortown.hackclub.com/).

## Download

Downloads are available in
[GitHub Releases](https://github.com/Not-A-Normal-Robot/hack-club-space-program/releases),
if I don't forget to add it.

## Building for Native

You'll need Cargo to build this:

```
cargo build
```

To run the built file:

```
cargo run
```

## Building for Web

You'll need Rustup and Deno to build this:

```
deno -P scripts/build-web.ts
```

NOTE: You'll need Cargo and Rustup to be in your `PATH` whilst running the build
script.

To do this, you can insert it manually:

```
export PATH="$PATH:$HOME/.cargo/bin" && deno -P scripts/build-web.ts
```

Or you can import `~/.cargo/env` before running the script:

```
~/.cargo/env && deno -P scripts/build-web.ts
```

### Compilation Features

Use the `trace` feature to enable trace logging.
