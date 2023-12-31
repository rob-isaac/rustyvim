# Rustyvim

A DIY editor inspired by [this guide](https://viewsourcecode.org/snaptoken/kilo/).

# Project Goals

This is mainly a pet project for myself to become more familiar with rust. As
such, I am hoping to maintain a minimal dependency list and implement most
things by myself.

I'm hoping to eventually implement most of the features that I use in neovim
both from its core and from plugins. I am also hoping to add something that
I find missing in neovim: good remote development support. Particularly, I want
to design this in such a way that puts all the buffer manipulation work on the
local client as possible, and only interacts with the remote machine for things
like external commands, LSP results, etc.

# Design

There will be a 3-teir separation-of-concerns:
- Tier1: Responsible for reading keys from the user and drawing application to the terminal.
- Tier2: Responsible for mutating the application state.
- Tier3: Responsible for interacting with the external state (filesystem, external commands, language servers, etc).

The goal of separating Tier1 and Tier2 is to allow multiple terminal instances
to share the same application state. The goal of separating Tier2 and Tier3 is
to allow Tier2 to run locally even while Tier3 runs remotely so that the
application updates quickly even over a slow connection (though things like
file-search and lsp results will still experience network lag).
