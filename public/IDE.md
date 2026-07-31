---
purpose: Scope project
pipeline:
  - Add features
  - Create build
  - Add to website
---

# IDE

An ambitious project to say the least

- [[#Roadmap]]
- [[#Onboarding]]
- [[#UI]]

# Roadmap

- [ ] Search Project
  - [ ] Add a tag system for favoring results and enabling them to reorder...
  - [ ] Make the algorithm visible
- [ ] cmd palette variants via
  - [ ] Hints for new layers/shortcuts/functionality/forks/panels until user has "explored"
  - [ ] New Gates of cmd-shift-p:
    - [x] cmd+shift+p
      - File Search only...
      - "up" shows previous commands but doesn't enable going "down" to select other similar commands which we're often looking for.
      - Add a "run" and "tag" prompt to build a LRU cache...
        - Sidebar of cmd palette where after running cmd palette they can then do "1" to execute.
        - And/or then
    - [ ] cmd+shift+p :
      - Vim... focus a group of commands via post fix key
      - Vim... focus a group of commands via post fix key
    - [ ] cmd+shift+p /
    - [ ] cmd+shift+p ;
    - [ ] cmd+shift+p -
    - [ ] cmd+shift+p =
    - [ ] cmd+shift+p [
    - [ ] cmd+shift+p ]
    - [ ] cmd+shift+p cmd+shift+p
  - [ ] UI Plan:
    - [ ] Bottom Row should have shortcuts showing how to change behavior
    - [ ] cmd-shift-p shift-1 to swap panels
    - [ ] left right arrows to cycle tabs
    - [ ] Should "filtering" be a thing? Because it limits, which is a principle.
      - [ ] We should enable 'sorting' or favoring via cmd palette though.

## Onboarding

A universe class IDE for the next 1000 years does what?
It helps keep the user focused on the goal while at the the same time training them to "be better".

- [Teach-1] [[Keymap]]
  - Intro UI
- [Teach-2] [[Estate]]
  - "Module 0"
    - Store cmd history: starts, cmds, executions, project stats, dirs, not just "graph resolution"
- [Teach-3] [[#Legend]]
  - What symbols mean what?
  - MD spec for now...
- [Teach-4] [Keymap]
- [Teach-5] [Keymap]
- [Teach-6] [Keymap]
- [Teach-7] [Keymap]

- [ ] Estate:

## UI

- FOLD
  - FM Section fold
  - Fold regions
- [ ] Tmux shortcut windows
  1. [shift shift][Chord]:
  2. [1|2|3][navigation]: Triggers left, bottom, right, dock focus when 1,2,3 pressed next.
  3. [1|2|3][ui]: Reveals numbered icon overlay whilst in "chord mode."

### Mnemonic Panel [[]]

- [ ] Browser

## Legend

- [Goal-1] [Concept]: Pipelines are choose your own phase, stage, step toward a goal for a concept.
- [Goal-2] [Keymap]: Syntax

### Key map variants

- **Editor Focus Chords**:
  - [cmd h]: Cycles prev sibling
  - [cmd l]: Cycles next sibling
  - [shift h]: Cycles prev cousin
  - [shift l]: Cycles next cousin
  - [shift shift h]: Focuses left
  - [shift shift l]: Focuses right
  - [cmd cmd h]: Toggles left dock
  - [cmd cmd l]: Toggles right dock
  - [cmd cmd j]: Toggles bottom dock
  - [cmd cmd k]: ...
  - [cmd n]: When editor focused focuses to tab n.
  - [shift n]: When editor focused focuses to pane n.
  - [shift shift-1]
  - [shift-? shift]
- TTL: Show bookmarks for 5 seconds with numbered activation icons: "1 h" and "1 l" for 1 in left dock and 1 in right respectively.
- TTL: Bookmarks

## Karabiner

VScode prevents modifier shortcuts so let's use Karabiner to alias them.

- [shift] as f13 press
- [shift shift] as f13 double tap
- [shift shift-*] as f13 hold with wild card
- [cmd] as f14 press
- [cmd cmd] as f14 double tap
- [cmd cmd-*] as f14 hold with wild card
- [shift shift-l] f14

<!--
    Bookmar Variant.
    Karabiner config file which I put here to know "what works".
    When I "left checked" for one shortcuts.
    - How to automate/sync?
 -->

```json
{
  "description": "Left Command tap → F13",
  "manipulators": [
    {
      "from": {
        "key_code": "left_command",
        "modifiers": { "optional": ["any"] }
      },
      "to": [
        {
          "key_code": "left_command",
          "lazy": true
        }
      ],
      "to_if_alone": [{ "key_code": "f13" }],
      "type": "basic"
    },
    {
      "from": {
        "key_code": "right_command",
        "modifiers": { "optional": ["any"] }
      },
      "to": [
        {
          "key_code": "right_command",
          "lazy": true
        }
      ],
      "to_if_alone": [{ "key_code": "f13" }],
      "type": "basic"
    },
    {
      "from": {
        "key_code": "left_shift",
        "modifiers": { "optional": ["any"] }
      },
      "to": [
        {
          "key_code": "left_shift",
          "lazy": true
        }
      ],
      "to_if_alone": [{ "key_code": "f14" }],
      "type": "basic"
    },
    {
      "from": {
        "key_code": "right_shift",
        "modifiers": { "optional": ["any"] }
      },
      "to": [
        {
          "key_code": "right_shift",
          "lazy": true
        }
      ],
      "to_if_alone": [{ "key_code": "f14" }],
      "type": "basic"
    }
  ]
}
```

## Obsidian level .md support

- [[vscode]] outline panel sucks
- [[Zed]] no configs
