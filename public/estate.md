---
purpose: North star for merging ideas
---

# Estate

Human Toolchain Paradigm
The human is the continuous thread. The tools exist to preserve and amplify their context across time.

**Problems**

- Knowledge is scattered across
  - directories
  - commits
  - branches
  - remotes
- People dont want to
  - think about fs
  -

**Solutions**

- Diverge
- Split
- Force

**Realization**

- Learn the last system.
- We don't want to make one release.
- We want to make many.

**Examples**

- Writings
  - Docs for the team
  - Blogs about experience on a project
  - Notes from university
- Projects
  - Built from memory, skills, experience,
    - Are technically "files" but are really "sums".
    - Youtube tutorials
    - Links to Repos you found on Github
    - Usually lists of all of the above.

**Goals**

- [[Semantic Workspace]]
  - Flat, Dynamic
    - Orathangonal?
    - A 'mono repo' with 2 different frameworks in same dir
    - Dynamic 'workshop' with all our tools & 'spot' or 'chair'(fs dir)
- [IDE]
- [Bookmarks]

---

## Semantic Workspace

A workspace that's

**Spec**

- init
  - public personal website from workship dir .estate init
- [Estate] Compliant
  - [Mod 0] scopes
  - maintained via registry.json
  - Outcome Dir
- []: Generated
  - ./outcomes
    - index.html
    - generated .md docs if they're writer.
    - generated bin if they're developers

**Enables**

- Doesn't change. Represents the result the user wants to keep "top of mind"

- UI projections via
- IDE
- Explicit
  - Mode Switch: "react", "rust"
    - Side effect:
      - .settings.json changed (tree view, dir search)
      - .gitignore
  - Derived: dev, debug,
  - .git for src control
  - .gitignore
  - .settings.json for tree view

## IDE

- Contact point
- Messenger
  - Suggest related youtube videos
  - - [Troubleshoot]
      - Reveal all those configs in sidebar.
      - Remove all config dirs from .settings.hide or config files
    - [Build] Reveal output dirs
    - [Architecture]
      - Enhanced Outline Panel
      - .ai dir auto changes

## Core/CLI/Process/Runtime

Core architecture for semantic workflow paradigms to be realized in other tools.

- IDEs
- LSPs
- Extensions

**Responsibilites**

- Builds
- Serves IDE
- Maintains recursive registry.json

**Enables**

- [[Semantic Bookmarks]]

## Semantic Bookmarks

- IDE Inserts
- Providers references. Sidebar? Popup?
- Enables recalling of ideas both up and down:
  - Label writing from a specific project, keeping them "close to source"
  - Save snippets from project
    - Enables "variants" we wouldn't necessarily want to commitor even have inside the IDE because it may be distracing to the "bigger overall goal"

### Spec

- Auto .settings file resolver. IDE independent
  - Enables quick switch between "modes"
    - Dev Hide .config files
      - .gitignore
      - .gitsubmodules
      - .prettierrc
      - .markdownlint.jsonc
      - .tsconfig.js
      - next.config.js

## .loi

---
Heading in same file Same-file heading link
---

- [[wikilinks]] Link to file
- [[wikilinks|Display Text]] Custom display text
- [[wikilinks#Heading]] Link to heading
- [[wikilinks#^block-id]] Link to block
- [[#Heading in same file]] Same-file heading link

![[wikilinks]] Embed full file
![[wikilinks#Heading]] Embed section
![[image.png]] Embed image
![[image.png|300]] Embed image with width

[[share-manifest-assets]]
