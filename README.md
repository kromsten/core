# Stargaze Core Libraries

### FORK-INFO 
Referencing local (latest) version of `sg-std` instead of publicly published one

Fixes issues with different version of `requires_stargaze` being injected by different versions of `sg-std`

<hr>

This repository contains the core contracts and libraries that are shared among all Stargaze protocols.

| Package                                                     | Description                                                                                      |
|-------------------------------------------------------------|--------------------------------------------------------------------------------------------------|
| [Stargaze Fair Burn](./contracts//fair-burn/README.md)      | Contract for fees and Developer Royalties.                                                       |
| [Stargaze Standard Library](./packages/sg-std/README.md)    | Common Stargaze libraries for interfacing with CosmWasm smart contracts.                         |
