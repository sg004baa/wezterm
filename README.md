# sg004baa/wezterm — personal fork

Personal fork of [wezterm/wezterm](https://github.com/wezterm/wezterm).

**Upstream base:** `05343b387` (upstream `main`, 2026-01-02)

## Custom modifications

- Floating Pane
    - Real terminal panes that float over the normal pane layout. Spawned via `SpawnCommandInNewFloatingPane` and toggled with `ToggleFloatingPane`.
- Floating Overlay
    - Unified styling config consumed by all four modal overlays (CommandPalette, CharSelector, PromptInputLine, InputSelector).
    ```lua
    config.floating_overlay = {
      width = '80%',
      height = '60%',
      bg_color = '#1e1e2e',
      padding = { left = 8, right = 8, top = 8, bottom = 8 },
    }
    ```
- fixes
    - Render loop freeze when closing workspaces
    - Win32 system backdrop stays visible while window is inactive
    - Scrollbar hit region heights corrected (`ScrollHit::thumb` field semantics)
    - `compute_background_rect_with_scrollbar` padding argument order corrected

---

# Wez's Terminal

<img height="128" alt="WezTerm Icon" src="https://raw.githubusercontent.com/wezterm/wezterm/main/assets/icon/wezterm-icon.svg" align="left"> *A GPU-accelerated cross-platform terminal emulator and multiplexer written by <a href="https://github.com/wez">@wez</a> and implemented in <a href="https://www.rust-lang.org/">Rust</a>*

User facing docs and guide at: https://wezterm.org/

![Screenshot](docs/screenshots/two.png)

*Screenshot of wezterm on macOS, running vim*

## Installation

https://wezterm.org/installation

## Getting help

This is a spare time project, so please bear with me.  There are a couple of channels for support:

* You can use the [GitHub issue tracker](https://github.com/wezterm/wezterm/issues) to see if someone else has a similar issue, or to file a new one.
* Start or join a thread in our [GitHub Discussions](https://github.com/wezterm/wezterm/discussions); if you have general
  questions or want to chat with other wezterm users, you're welcome here!
* There is a [Matrix room via Element.io](https://app.element.io/#/room/#wezterm:matrix.org)
  for (potentially!) real time discussions.

The GitHub Discussions and Element/Gitter rooms are better suited for questions
than bug reports, but don't be afraid to use whichever you are most comfortable
using and we'll work it out.

## Supporting the Project

If you use and like WezTerm, please consider sponsoring it: your support helps
to cover the fees required to maintain the project and to validate the time
spent working on it!

[Read more about sponsoring](https://wezterm.org/sponsor.html).

* [![Sponsor WezTerm](https://img.shields.io/github/sponsors/wez?label=Sponsor%20WezTerm&logo=github&style=for-the-badge)](https://github.com/sponsors/wez)
* [Patreon](https://patreon.com/WezFurlong)
* [Ko-Fi](https://ko-fi.com/wezfurlong)
* [Liberapay](https://liberapay.com/wez)
