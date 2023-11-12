# PSP34 Token
Repo stores an example of a compliant implementation of the PSP34
community standard [1] in plain ink. This is mostly inspired by the work
of openbrush team [2] and the focus was to remove the dependency on
their internal libraries in favour of ink's alternatives. Shout-out to
the paritytech peeps too for their amazing up-to-date examples
repository. [3]

- [1] https://github.com/w3f/PSPs/blob/master/PSPs/psp-34.md
- [2] https://github.com/Brushfam/openbrush-contracts
- [3] https://github.com/paritytech/ink-examples/

## How to use this repository

To use this crate please add the following line in your project's `Cargo.toml`:
```
psp34 = { git = "https://github.com/Just-Ape-Studios-Admin/just-ape-studios-contracts.git", default-features = "false" }
```

The contents of this repository can be used in following ways:

### 1. Ready to use contract

The file [`lib.rs`][lib] contains a ready to use implementation of basic PSP34 token contract (extended with PSP34Metadata). To use it, please check out this repository and compile its contents using [`cargo-contract`][cargo-contract] with the `"contract"` feature enabled:
```
$ cargo contract build --release --features "contract"
```
### 2. Cross contract calling with traits

The `PSP34` trait contains all the methods defined in the PSP34 standard. The trait can be used together with ink!'s [`contract_ref`][contract_ref] macro to allow for convenient cross-contract calling.

In your contract, if you would like to make a call to some other contract implementing the PSP34 standard, all you need to do is:
```
use ink::contract_ref;
use ink::prelude::vec::vec;
use psp34::PSP34;

let mut token: contract_ref!(PSP34) = other_contract_address.into();

// Now `token` has all PSP34 methods
let balance = token.balance_of(some_account);
token.transfer(to, id, vec![]); // returns Result<(), PSP34Error>
```

The same method can be used with other traits (`PSP34Metadata`, `PSP34Enumerable`, `PSP34Mintable`) defined in this crate. See the contents of [`traits.rs`][traits] for details.

### 3. Custom implementation of PSP34 logic with `PSP34Data`

The `PSP34Data` class can be used to extend your contract with PSP34 token logic. In other words, you can easily build contracts that implement PSP34 interface alongside some other functionalities defined by the business logic of your project.


[lib]: ./lib.rs
[traits]: ./traits.rs
[ink]: https://use.ink
[substrate]: https://substrate.io
[cargo-contract]: https://github.com/paritytech/cargo-contract
[erc20]: https://ethereum.org/en/developers/docs/standards/tokens/erc-20/
[PSP34]: https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md
[contract_ref]: https://paritytech.github.io/ink/ink/macro.contract_ref.html

## Caveats

1. PSP34' standard defines that the type `Id` can be at max a `u128`,
   meaning you can mint up to 2^128 - 1 tokens. Yet, some messages
   don't account for that, e.g. `balance_of` is set to return an
   `u32`, which could end up causin' an integer overflow if the same
   owner has a biiig chunk of the supply. So just be aware of that <3
2. No built-in extra stuff. If you wanna go custom, like charge for
   each mint, mint based on an allow-list, check only the contract
   owner can call transfer, and so on; you gotta do it yourself... or
   send a gig my way.