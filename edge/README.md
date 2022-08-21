# Mandelatar - Cloudflare Worker

This is a rust worker implementation of the Mandelatar image generation backend, for quickly generating images on Cloudflare's edge network.

## Wrangler docs

See the [wrangler docs](./wrangler_docs.md) page for a quick overview of how to run and deploy the worker implemented here.

## Benchmarks

Because Cloudflare's free tier currently limits CPU time to 10 ms, it's especially important to track performance for this part of the system. This repo uses [Criterion](https://docs.rs/criterion/latest/criterion/) benchmarks to evaluate performance of heavy parts of the image generation process.

Running these benchmarks is just a matter of running:
```
$ cargo bench
```
