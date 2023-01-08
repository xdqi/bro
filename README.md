bro
===================
# Introduction
One project in Rust that allows users to choose their web browser when clicking a link is [bro](https://github.com/xdqi/bro).

`bro` is a command-line utility that allows users to open links in their preferred web browser. It is designed to be a simple and lightweight alternative to similar utilities like `xdg-open`.

To use `bro`, users can simply type `bro <link>` in their terminal, and bro will prompt them to choose their preferred web browser from a list of installed browsers. They can then select their desired browser and `bro` will open the link in that browser.

`bro` is a good choice for users who want a lightweight and customizable way to open links in their preferred web browser from the command line.

*generated with ChatGPT*

# Introduction, really

A replacement of Browser Chooser 2 in Rust with support for smart browser/profile detection.

Browser choosing rules are defined in order of precedence.

Currently only Windows is supported.

Browsers that supports detection: Chrome, Firefox

# Installation

1. `git clone https://github.com/xdqi/bro`

2. `cargo build --release`
