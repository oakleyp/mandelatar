# Mandelatar

An app for generating random images of the Mandelbrot set that can be referenced by a single string, with no DB required - "Like Gravatar, but with the Mandelbrot set".

This is a port of my original repo ([here](https://github.com/oakleyp/mandelatar)) to a version that runs on Cloudflare's edge network using [Workers](https://developers.cloudflare.com/workers/).

## Pros/Cons vs. Original

Pros:
- No hosting cost (10ms execution time per request is currently free)
- Less latency from pretty much every conceivable location on earth outside of directly next to the single Digital Ocean droplet where the original is hosted (but maybe there too)
- Overall near-instant load times compared to previous version, even using only a single thread (this could be due to network, or resources available to the worker process, or likely both; I have not benchmarked for this yet)

Cons:
- No concurrency (workers only run WASM)
- Some crates unsupported, CF workers crate is still young - adding some complexity beyond what's here may require reinventing a wheel or two

## Live Demo

There is no front end yet - but opening the following link will redirect you to a random image url, which you can use anywhere you might want a 300x300 image:
https://mandelatar-edge.oakleypeavler.com/api/v1/random

There are theoretically an infinite number of images due to the nature/magic of the Mandelbrot set, and they are generated deterministically based on the string in the resulting redirect URL. As a result, there is no per-user application state and no need to run a database, at the cost of a small amount of processing time on the server side (and additional image load time on the client side).

## Available Query Param Options

Currently there is one available render configuration param: `?overlay=profile`. Using this option will add a "user profile" overlay to the rendered output, e.g. https://mandelatar-edge.oakleypeavler.com/api/v1/random?overlay=profile

Additional config params will be documented here as there are added.

| Param | Description | Possible Values|
| ---- | ---- | --- |
| overlay | Renders a preset overlay image in the output | profile |

## Examples

![Image 1](https://mandelatar-edge.oakleypeavler.com/api/v1/img/WAIAAAAAAABYAgAAAAAAAHPdINacevO_XuBkOef41z8ICQGBYsbuv7P95XaoYMc_DupC8js25D9p35AB.png)

![Image 2](https://mandelatar-edge.oakleypeavler.com/api/v1/img/WAIAAAAAAABYAgAAAAAAAJiTLYv6Z_G_FgGIwF2J0T8En9PILp7wv8RxDcciRs4_iCpEuUkewz5_6-oB.png?overlay=profile)

![Image 3](https://mandelatar-edge.oakleypeavler.com/api/v1/img/WAIAAAAAAABYAgAAAAAAAGsGaBqMXfG_j3F3yJAM0j9BCvNHJpXwv3Q291IWV88_NSl0VwTW8D0BTLMA.png)

![Image 4](https://mandelatar-edge.oakleypeavler.com/api/v1/img/WAIAAAAAAABYAgAAAAAAAOYjkn24sPG_oJTPsYQ50T-HEGCz0NzwvwPv5kQihc0_kt9ERNBbgj-sCzcC.png)