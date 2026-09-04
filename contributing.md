# Contributing

Improvements to performance or code quality, and bug fixes are always welcome.
Feature additions should be talked through with the maintainer first.

## Project overview

- [`cheat/src/cs2`](/cheat/src/cs2) contains all the game-specific code
  - [`entity`](/cheat/src/cs2/entity) contains everything relating to in-game entities, like players, grenades, and weapons
  - [`features`](/cheat/src/cs2/features) should be self-explanatory
- [`cheat/src/ui`](/cheat/src/ui) contains both the gui and overlay code
- [`cheat/src/parser`](/cheat/src/parser) contains a bvh implementation, for fast visibility lookups
- [`cheat/src/os`](/cheat/src/os) contains low-level os interactions, like reading/writing memory and mouse input
- [`server`](/server) is the web-radar TCP/WebSocket backend
- [`radar`](/radar) is the Svelte radar frontend
- [`shared`](/shared) holds types shared between cheat and server

## What i won't merge

- **Giant PRs**: PRs that touch lots of different places are hard to sift through, and should be split into multiple, smaller PRs.
- **Anything AI-generated**: see below.

## LLMs

Usage of LLMs is not wanted.
If you cannot code yourself, please do not open PRs here.
I wish to maintain a well-organized and small codebase, and LLMs are not very good at doing that.
