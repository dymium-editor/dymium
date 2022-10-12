# Architecture

This file serves as a brief description of the overall architecture of the `dymium` editor.
Individual components may have their own `ARCHITECTURE.md` files for more in-depth descriptions
there.

## Responsibilities of crates

- `dymium-term` -- Terminal interaction library for `dymium`. Has no local dependencies.
  - Also contained in this crate's directory is `capdata.yaml`, which stores the default terminal
    capabilities dataset
