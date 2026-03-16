# dioxus-showcase

The user-facing facade crate.

This crate is the public API consumers import in application code. It re-exports the core types and macro surface, defines the `StoryArg` and `StoryProps` traits, supplies default argument generators for common Dioxus-friendly types, and provides the provider-wrapping preview component used by generated runtimes.

## Use It For

- `dioxus_showcase::prelude::*`
- `StoryArg`, `StoryArgs`, `StoryProps`, and `StoryVariant`
- `GeneratedStory` and `ShowcaseStoryFactory`
- `StoryPreviewContent`

## Mental Model

The macros generate helper symbols against this crate’s traits and types. The CLI later discovers the annotated functions and emits a showcase app that imports those helper symbols back out of the entry crate.

The complete API map is in [`../../docs/code-reference.md`](../../docs/code-reference.md).
