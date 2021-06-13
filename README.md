# Stocksh

Stocksh is console oriented service for stock prices. The output uses terminal-oriented ANSI-sequences
for console HTTP clients to provide clean and concise look.

## Usage

Stocksh isn't hosted anywhere yet, so you have to:

```
> IEX_CLOUD_TOKEN="your_token" cargo run
> curl localhost:8080/quote/NET
NET $93.97 +0.53 (+0.567)
```

## Road map

- [ ] - custom format?
- [ ] - support for indexes, ETFs, sectors?

