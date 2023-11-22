# PSP34 Example
This example shows a basic implementation of the PSP34 standard in this repository.

## Usage
Ensure to import the repository in your Cargo.toml file thus:
```
  psp34 = { git = "https://github.com/just-ape-studios/just-ape-studios-contracts.git", default-features = false }
```
Add support for "std" within the "std" key of your Cargo.toml file where you probably have "ink/std" included.
```
  std = [
    "ink/std",
    "scale/std",
    # Other keys go here...
    "psp34/std" #This enables the PSP34 std
  ]
```

These are important to get your PSP34 logic up and running. Feel free to make PRs or submit issues.

Goodluck!
