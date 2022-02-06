# forwardemail-webhook-rs

This is a webhook that accepts e-mails as HTTP POST body as implemented by [forwardemail.net](https://forwardemail.net/en/faq#do-you-support-webhooks).

# Deploy

## TODO

* make sure the spool directory exists (readable by service user)
* register with Caddy (requires restart)
* systemd unit file

# Release

There is a Concourse pipeline in `ci`. It creates a draft GitHub release for every tag:

```command
$ git tag -a v1.0.0-pre1 -m "Preparing release v1.0.0"
$ git push --follow-tags
```

# Development

* Use `cargo watch -x 'run'` to iterate
