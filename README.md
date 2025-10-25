# Hide & Seed

---

## RUST

*Simple app for combining/extracting files inside another file, usually images.*

---

## Overview

**Hide & Seed** is a simple Rust application for combining (hiding) and extracting files within another file, typically images. This tool is useful for basic steganography, allowing you to conceal multiple secret files inside a cover file and later extract them by their original names.

---

## Features

- Combine single or multiple files into a single image or cover file.
- Full encryption of the secret files before hiding. 
- Double Securing the hidden part of the final file, using password.
- Extract hidden files by their actual names (after entering correct password).
- User-friendly interactive menu-driven interface (not just command-line arguments).

---

## Install

```sh
sudo dpkg -i ./pkg/hidenseed-package.deb
```

---

## Usage

1. Run `hidenseed` from your terminal.
2. Use the interactive menu to select actions such as hiding files or extracting them.
3. Follow prompts to choose files and options as needed.

---

## Status

*Hide & Seed* is under active development. See the TODOs below for planned features and improvements.

---

## TODOs
- Maybe create a GUI?