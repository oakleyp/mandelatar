# Mandelatar

An app for generating random images of the Mandelbrot set that can be referenced by a single string, with no DB required - "Like Gravatar, but with the Mandelbrot set".

This currently runs as a system combining Cloudflare [Workers](https://developers.cloudflare.com/workers/) for generating images (where possible) at the near edge of the client, and a Digital Ocean Droplet hosting the frontend and generating images too expensive to be run on workers.

## Live Demo / Production Instance

[mandelatar.com](https://mandelatar.com)

There are theoretically an infinite number of images due to the nature/magic of the Mandelbrot set, and they are generated deterministically based on the string in the resulting redirect URL. As a result, there is no per-user application state and no need to run a database, at the cost of a small amount of processing time on the server side (and additional image load time on the client side).

## Running locally

### Via docker-compose

Given you have docker installed, this should just be a matter of running the following in the root of the repo:  
`$ docker-compose up -d`

This sets up a docker stack with the rust backend running behind a Traefik proxy (although locally, the proxy can be bypassed).

Once the stack is running, you should be able to reach the service locally at through Traefik at:  
`localhost/api/v1/random`

Or, you can reach the service directly at:
`localhost:8080/api/v1/random`

You can reach the Traefik dashboard at:
`localhost:8090`

**Note:** you can change these ports by modifying the services' port mappings in the `docker-compose.override.yml` file.

### Directly on your machine

Given you have the rust toolchain (edition 2021) installed, you can build the debug version by running the following in the `backend/` directory:

`$ cargo build`

Then start the server via:

`$ [ENV VARS] ./target/debug/mandelatar`

For example:  

`$ RUST_LOG=debug MANDELATAR_SERVER_PORT=8080 ./target/debug/mandelatar`

The following env vars (shown here with the defaults) are used for configuration at runtime:

```
MANDELATAR_SERVER_ADDR=127.0.0.1
MANDELATAR_SERVER_PORT=8080
MANDELATAR_CORS_ORIGINS="..." # Change these to your own origin servers
# Some valid options: [error|warn|info|debug|trace]
RUST_LOG=error
```

See the [env_logger](https://docs.rs/env_logger/latest/env_logger/) docs for more possible values of `RUST_LOG`.

## Deploying in production

The architecture used here follows the guidelines of [dockerswarm.rocks](https://dockerswarm.rocks). If you want to run this in production, you'll need to first set up a root traefik proxy as described there. This system will be served through a second proxy If all else is configured properly, deploying is then just a matter of running the `scripts/deploy.sh` script on your server with the proper env vars filled in, e.g:

```
sudo DOMAIN=mandelatar.com TRAEFIK_TAG=mandelatar.com STACK_NAME=mandelatar-com TAG=prod bash ./scripts/deploy.sh
```

## Setting up the Cloudflare Worker:

From the `edge/` directory, you can use the [Wrangler Docs](./edge/wrangler_docs.md) for general information on configuration and deployment of the worker via wrangler.

## WIPs

- Tests
- Better mandelbrot variety
- Custom image resolutions
- Frontend Improvements

## API

Assuming the host is https://mandelatar.com:

- Requests on the path `/api/v1/...` are routed directly to the droplet server
- Requests on the path `/i1/...` are routed to the worker process, but should fail over to the droplet server when the worker reaches free tier limits, or the image requested is determined to be too complicated (expensive) for the worker to handle

### Available Query Param Options

Currently there is one available render configuration param: `?overlay=profile`. Using this option will add a "user profile" overlay to the rendered output, e.g. https://mandelatar.com/api/v1/random?overlay=profile

Additional config params will be documented here as there are added.

| Param | Description | Possible Values|
| ---- | ---- | --- |
| overlay | Renders a preset overlay image in the output | profile |

## Examples

![Image 1](https://mandelatar.com/api/v1/img/WAIAAAAAAABYAgAAAAAAAHPdINacevO_XuBkOef41z8ICQGBYsbuv7P95XaoYMc_DupC8js25D9p35AB)

![Image 2](https://mandelatar.com/api/v1/img/WAIAAAAAAABYAgAAAAAAAJiTLYv6Z_G_FgGIwF2J0T8En9PILp7wv8RxDcciRs4_iCpEuUkewz5_6-oB?overlay=profile)

![Image 3](https://mandelatar.com/api/v1/img/WAIAAAAAAABYAgAAAAAAAGsGaBqMXfG_j3F3yJAM0j9BCvNHJpXwv3Q291IWV88_NSl0VwTW8D0BTLMA)

![Image 4](https://mandelatar.com/api/v1/img/WAIAAAAAAABYAgAAAAAAAOYjkn24sPG_oJTPsYQ50T-HEGCz0NzwvwPv5kQihc0_kt9ERNBbgj-sCzcC)
