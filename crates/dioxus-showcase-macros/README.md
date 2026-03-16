# dioxus-showcase-macros

The compile-time engine behind the showcase prototype.

This proc-macro crate turns user-authored Dioxus functions into hidden renderer symbols, story factories, constructor helpers, and provider wrappers. It does not discover files on disk and it does not run the app. Its job is to make the annotated code callable by the generated showcase runtime.

## Macro Surface

- `#[showcase]`: register a component as a story source.
- `#[story]`: register a named story function.
- `#[provider]`: register a wrapper applied around every story.
- `#[derive(StoryProps)]`: derive default `StoryArg` and `StoryProps` implementations from `Default`.

## Design Notes

- Zero-arg stories/components render directly.
- Aggregate `props` arguments expand through `StoryProps`.
- Multi-arg functions get a generated controls shell when supported control types are detected.
- Provider wrappers always preserve the original function and synthesize non-`children` args from `StoryArg`.

For expansion details, read [`../../docs/code-reference.md`](../../docs/code-reference.md).
