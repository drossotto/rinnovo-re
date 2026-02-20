# rinnovo-sdk

High-level Python SDK for the **Rinnovo Representation Engine (RE)**.

Rinnovo Binary (RNB) is a segment-based, memory-mappable binary format for
biological data. The **Representation Engine** sits on top of that format and
exposes a semantic view of the data: objects, types, attributes, relationships,
and query kernels. The engine is implemented in Rust; `rinnovo-sdk` is the
Python front-end to that engine.

While the SDK can emit and manage `.rnb` artifacts, its primary role is to let
you work with the *representation* rather
than with raw files.

## Installation

For development from this repository:

```bash
cd rinnovo_sdk
poetry install
