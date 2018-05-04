# efloat

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Documentation is available at https://docs.rs/efloat

**efloat** is a component within the Siege Engine MMO game engine, but should be
generally useful outside of that context.

The Siege Engine is an MMO game engine on the Vulkan API written in the Rust language.

efloat provides floating a point type that remembers how far off it might be from
the actual precise value, based upon its history. It keeps and upper and lower error
bound internally, and you can check those with function calls.

