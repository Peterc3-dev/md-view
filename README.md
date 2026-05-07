# md-view

Terminal Markdown previewer with live-rendered headings, code blocks, TOC navigation, search, and file watching.

## Features

- Renders Markdown with styled headings, bold, italic, code blocks, lists, and blockquotes
- Table of contents panel auto-generated from headings -- jump to any section
- Incremental `/` search with `n`/`N` match cycling
- `--watch` mode: auto-reloads on file save (via `notify` inotify)
- Reads from file or stdin pipe
- Vim-style scrolling (`j`/`k`, `Ctrl-D`/`Ctrl-U`, `g`/`G`, PageUp/PageDown)
- Toggle raw Markdown source with `h`
- Phosphor-green terminal aesthetic

## Install

```
cargo build --release
cp target/release/md-view ~/.local/bin/
```

## Usage

```bash
md-view README.md              # view a file
md-view --watch notes.md       # live-reload on save
curl -s URL | md-view           # pipe from stdin
```

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll down / up |
| `Ctrl-D` / `Ctrl-U` | Half-page down / up |
| `g` / `G` | Top / bottom |
| `PageUp` / `PageDown` | Page scroll |
| `/` | Enter search mode |
| `n` / `N` | Next / previous match |
| `Tab` | Toggle table of contents |
| `Enter` (in TOC) | Jump to heading |
| `h` | Toggle raw Markdown source |
| `q` / `Esc` | Quit |

Built with Rust + ratatui.
