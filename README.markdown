# forwardemail-webhook-rs

This is a webhook that accepts e-mails as HTTP POST body as implemented by [forwardemail.net](https://forwardemail.net/en/faq#do-you-support-webhooks).

# Build

## Once

```command
$ rustup target add x86_64-unknown-linux-gnu
$ rustup toolchain install stable-x86_64-unknown-linux-gnu
```

## Release

```command
$ cargo build --release --target x86_64-unknown-linux-gnu
```

# Development

* Use `cargo watch -x 'run'` to iterate
