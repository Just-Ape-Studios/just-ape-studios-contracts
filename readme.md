## Abstract

Repo stores an example of a compliant implementation of the PSP34
community standard [1] in plain ink. This is mostly inspired by the work
of openbrush team [2] and the focus was to remove the dependency on
their internal libraries in favour of ink's alternatives. Shout-out to
the paritytech peeps too for their amazing up-to-date examples
repository. [3]

This ain't a library that you can import into your code and
extend. You better off cloning it, checking which extensions you need
and then implementing what's missing on top of it.

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
3. No built-in extra stuff. If you wanna go custom, like charge for
   each mint, mint based on an allow-list, check only the contract
   owner can call transfer, and so on; you gotta do it yourself... or
   send a gig my way ((ΦωΦ)).
