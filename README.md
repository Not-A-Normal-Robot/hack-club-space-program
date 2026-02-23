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

### Dependencies (they need to be in PATH!)

- Rustup
- Cargo
- Deno
- Java

### Build Script

```
deno -P scripts/build-web.ts
```

### Compilation Features

Use the `trace` feature to enable trace logging.
