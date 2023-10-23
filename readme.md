## Summary
**These smart contracts were written in plain ink!.**

Why use these smart contracts?
- Smart contracts undergo rounds of security audits
- Standards can help simplify development

Token Standards available:
- **PSP34** - Non-Fungible Token (*ERC721 equivalent*) with extensions

### PSP-34 Non-Fungible Token Standard in Plain ink!
* PSP-34 OpenBrush Implementation [OpenBrush|https://github.com/Brushfam/openbrush-contracts/tree/main/contracts/src/token/psp34]

This compliant implementation of the PSP34
community standard [1] is mostly inspired by the work
of openbrush team [2].
The focus was to remove the dependency on
their internal libraries in favor of ink's alternatives. Shout-out to
the paritytech peeps too for their amazing up-to-date examples
repository. [3]

**Currently, the smart contract does not act as a library and cannot be extended.**
For easier use, please clone the repo, check which extensions you need
and then implement what's missing on top of it.

- [1] https://github.com/w3f/PSPs/blob/master/PSPs/psp-34.md
- [2] https://github.com/Brushfam/openbrush-contracts
- [3] https://github.com/paritytech/ink-examples/

## Caveats

1. PSP34' standard defines that the type `Id` can be at max a `u128`,
   meaning you can mint up to 2^128 - 1 tokens. Yet, some messages
   don't account for that, e.g. `balance_of` is set to return an
   `u32`, which could end up causin' an integer overflow if the same
   owner has a biiig chunk of the supply. So just be aware of that <3
2. There's no spec of whether the allowances of a token should be cleared
   on a transfer or not. On this impl they are cleared.
3. Tf is the `data` field in `transfer()` for you may ask? Nobody knows,
   openbrush's impl doesn't use it and neither do I, it's there for ABI
   compatibility.
4. If you require custom features, such as charge per mint, allow-list based mints, validate contract owner permissions, and so on; you must implement these features.
