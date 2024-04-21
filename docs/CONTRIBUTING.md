# Contributing

## How to contribute

1. Fork repo and create a new topic branch
2. Make changes
3. Ensure it compiles and passes tests using

```bash
cargo build
cargo test
```

4. Auto format the code using `rustfmt` or a tool that integrates it (such as
   `rust-analyzer` or some IDE).
5. Make small atomic compilable working commits. Do NOT use "Conventional
   Commits" for the commit title. Instead just directly write what was changed
without any prefixes. Write it in the imperative tense and use the
["50/72" rule](https://stackoverflow.com/a/11993051)
6. Push commits to the created topic branch in your repo.
7. Open a PR, wait for review.

## How to create a new Action

Use the [telegram action](../src/actions/telegram.rs) as a template. You need to
implement the `Action` trait for your action and add the necessary hooks in
[actions/mod.rs](../src/actions/mod.rs).
