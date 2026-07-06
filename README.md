## About this fork

This is a fork of [yazi](https://github.com/sxyazi/yazi) that extends the plugin
API and file-operation scheduler so that **Lua plugins can intercept, inspect,
and resolve paste operations on a per-item basis**. The upstream `paste` command
operates on the entire yanked set as a batch and silently appends `_1`, `_2`
suffixes to conflicting names. This fork adds the building blocks plugins need
to make per-item decisions — prompting the user, renaming on the fly, or
replacing/merging selectively — before any file is written.

### Changes

**1. Expose the pick dialog to Lua — `ya.pick()`**

The pick dialog already existed internally for yazi's own commands but was not
reachable from plugin code. A new `ya.pick({ title = …, items = { … } })`
binding (`yazi-plugin/src/utils/layer.rs`, `utils.rs`) lets any Lua plugin
present an interactive choice list and receive the user's selection
asynchronously. A companion `PickCfg::with_title()` builder
(`yazi-config/src/popup/options.rs`) allows the caller to override the dialog
title.

**2. Expose yanked URLs to Lua — `cx.yanked:urls()`**

The `Yanked` user-data now exposes a `:urls()` method
(`yazi-actor/src/lives/yanked.rs`) so plugins can read the full list of yanked
or cut paths and inspect them before deciding what to do.

**3. Per-item paste infrastructure — `paste_resolved` actor + parser**

A new `paste_resolved` command (`yazi-parser/src/mgr/paste_resolved.rs`,
`yazi-actor/src/mgr/paste_resolved.rs`) accepts a list of `{ from, to,
overwrite, replace }` items and dispatches each one individually. Unlike the
existing `paste` command — which copies/cuts the entire yank register to the
current directory — `paste_resolved` lets the caller specify an explicit
destination for every single item, choose whether to overwrite an existing
target, and choose whether to *replace* (delete-then-copy) or *merge* into an
existing destination. Registered in the executor (`yazi-fm/src/executor.rs`),
the Spark enum (`yazi-parser/src/spark/spark.rs`), and both module index files.

**4. Per-item scheduler methods — `file_copy_one` / `file_cut_one`**

`yazi-core/src/tasks/file.rs` gains `file_copy_one()` and `file_cut_one()`
that accept a single source URL, destination URL, force flag, and replace flag.
These are thin wrappers over the scheduler that feed one file at a time rather
than iterating over the whole yank register.

**5. Replace mode in the file scheduler**

`FileInCopy` and `FileInCut` (`yazi-scheduler/src/file/in.rs`) gain a `replace`
field. When set, the destination is deleted in its entirety *before* the
copy/cut begins (`yazi-scheduler/src/file/file.rs`), providing true
"replace directory" semantics — files that exist only at the destination are
removed, unlike the existing `force=true` merge behaviour which preserves them.
The field is propagated through `traverse.rs` and surfaced via
`Scheduler::file_copy_replace()` (`yazi-scheduler/src/scheduler.rs`) and
`FileInCut::with_replace()`.

**6. Pick dialog layout**

The default `open_offset` for the pick dialog (`yazi-config/preset/yazi-default.toml`)
was increased to accommodate a longer list of choices without truncation.

### Tests

```sh
cargo test --lib -p yazi-parser paste_resolved   # 5 parser tests
cargo test --lib -p yazi-config pick_cfg          # PickCfg::with_title test
```

### Example plugin

These changes are general-purpose and can be used by any plugin. An example
is the [paste-dialog](https://github.com/chodak166/yazi-paste-dialog) plugin,
which uses `ya.pick()` and `paste_resolved` together to show an interactive
conflict-resolution dialog when pasting.

---

<div align="center">
	<sup>Special thanks to:</sup><br>

| <a href="https://go.warp.dev/yazi" target="_blank"><img alt="Warp sponsorship" width=350 src="https://github.com/warpdotdev/brand-assets/blob/main/Github/Sponsor/Warp-Github-LG-02.png"><br><b>Warp, built for coding with multiple AI agents</b><br><sup>Available for macOS, Linux and Windows</sup></a> |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |

</div>

## Yazi - ⚡️ Blazing Fast Terminal File Manager

Yazi (means "duck") is a terminal file manager written in Rust, based on non-blocking async I/O. It aims to provide an efficient, user-friendly, and customizable file management experience.

💡 A new article explaining its internal workings: [Why is Yazi Fast?](https://yazi-rs.github.io/blog/why-is-yazi-fast)

- 🚀 **Full Asynchronous Support**: All I/O operations are asynchronous, CPU tasks are spread across multiple threads, making the most of available resources.
- 💪 **Powerful Async Task Scheduling and Management**: Provides real-time progress updates, task cancellation, and internal task priority assignment.
- 🖼️ **Built-in Support for Multiple Image Protocols**: Also integrated with Überzug++ and Chafa, covering almost all terminals.
- 🌟 **Built-in Code Highlighting and Image Decoding**: Combined with the pre-loading mechanism, greatly accelerates image and normal file loading.
- 🔌 **Concurrent Plugin System**: UI plugins (rewriting most of the UI), functional plugins, custom previewer/preloader/spotter/fetcher; Just some pieces of Lua.
- ☁️ **Virtual Filesystem**: Remote file management, custom search engines.
- 📡 **Data Distribution Service**: Built on a client-server architecture (no additional server process required), integrated with a Lua-based publish-subscribe model, achieving cross-instance communication and state persistence.
- 📦 **Package Manager**: Install plugins and themes with one command, keeping them up-to-date, or pin them to a specific version.
- 🧰 Integration with ripgrep, fd, fzf, zoxide
- 💫 Vim-like input/pick/confirm/which/notify component, auto-completion for cd paths
- 🏷️ Multi-Tab Support, Cross-directory selection, Scrollable Preview (for videos, PDFs, archives, code, directories, etc.)
- 🔄 Bulk Rename/Create, Archive Extraction, Visual Mode, File Chooser, [Git Integration](https://github.com/yazi-rs/plugins/tree/main/git.yazi), [Mount Manager](https://github.com/yazi-rs/plugins/tree/main/mount.yazi)
- 🎨 Theme System, Mouse Support, Drag and Drop, Trash Bin, Custom Layouts, CSI u, OSC 52
- ... and more!

https://github.com/sxyazi/yazi/assets/17523360/92ff23fa-0cd5-4f04-b387-894c12265cc7

## Project status

Public beta, can be used as a daily driver.

Yazi is currently in heavy development, expect breaking changes.

## Documentation

- Usage: https://yazi-rs.github.io/docs/installation
- Features: https://yazi-rs.github.io/features

## Discussion

- Discord Server (English mainly): https://discord.gg/qfADduSdJu
- Telegram Group (Chinese mainly): https://t.me/yazi_rs

## Image Preview

| Platform                                                                     | Protocol                               | Support                                |
| ---------------------------------------------------------------------------- | -------------------------------------- | -------------------------------------- |
| [kitty](https://github.com/kovidgoyal/kitty) (>= 0.28.0)                     | [Kitty unicode placeholders][kgp]      | ✅ Built-in                            |
| [iTerm2](https://iterm2.com)                                                 | [Inline images protocol][iip]          | ✅ Built-in                            |
| [WezTerm](https://github.com/wez/wezterm)                                    | [Inline images protocol][iip]          | ✅ Built-in                            |
| [Konsole](https://invent.kde.org/utilities/konsole)                          | [Kitty old protocol][kgp-old]          | ✅ Built-in                            |
| [foot](https://codeberg.org/dnkl/foot)                                       | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [Ghostty](https://github.com/ghostty-org/ghostty)                            | [Kitty unicode placeholders][kgp]      | ✅ Built-in                            |
| [Windows Terminal](https://github.com/microsoft/terminal) (>= v1.22.10352.0) | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [st with Sixel patch](https://github.com/bakkeby/st-flexipatch)              | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [Warp](https://www.warp.dev) (macOS/Linux only)                              | [Inline images protocol][iip]          | ✅ Built-in                            |
| [Tabby](https://github.com/Eugeny/tabby)                                     | [Inline images protocol][iip]          | ✅ Built-in                            |
| [VSCode](https://github.com/microsoft/vscode)                                | [Inline images protocol][iip]          | ✅ Built-in                            |
| [Rio](https://github.com/raphamorim/rio) (>= 0.3.9)                          | [Kitty unicode placeholders][kgp]      | ✅ Built-in                            |
| [Black Box](https://gitlab.gnome.org/raggesilver/blackbox)                   | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [Bobcat](https://github.com/ismail-yilmaz/Bobcat)                            | [Inline images protocol][iip]          | ✅ Built-in                            |
| X11 / Wayland                                                                | Window system protocol                 | ☑️ [Überzug++][ueberzug] required      |
| Fallback                                                                     | [ASCII art (Unicode block)][ascii-art] | ☑️ [Chafa][chafa] required (>= 1.16.0) |

See https://yazi-rs.github.io/docs/image-preview for details.

<!-- Protocols -->

[kgp]: https://sw.kovidgoyal.net/kitty/graphics-protocol/#unicode-placeholders
[kgp-old]: https://github.com/sxyazi/yazi/blob/main/yazi-adapter/src/drivers/kgp_old.rs
[iip]: https://iterm2.com/documentation-images.html
[sixel]: https://www.vt100.net/docs/vt3xx-gp/chapter14.html
[ascii-art]: https://en.wikipedia.org/wiki/ASCII_art

<!-- Dependencies -->

[ueberzug]: https://github.com/jstkdng/ueberzugpp
[chafa]: https://hpjansson.org/chafa/

## Special Thanks

<img alt="RustRover logo" align="right" width="200" src="https://resources.jetbrains.com/storage/products/company/brand/logos/RustRover.svg">

Thanks to RustRover team for providing open-source licenses to support the maintenance of Yazi.

Active code contributors can contact @sxyazi to get a license (if any are still available).

## License

Yazi is MIT-licensed. For more information check the [LICENSE](LICENSE) file.
