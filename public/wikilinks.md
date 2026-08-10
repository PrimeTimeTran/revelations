---
purpose: review
lifetime: vscode-extension
url: https://github.com/ltvan/markdown-wiki-links
---

# Heading 1

- [[kb]] Link to file
- [[kb|Display Text]] Custom display text
- [[kb#Heading]] Link to heading
- [[kb#^my-block-id]] Link to block
- [[ownership|Section]] Same-file heading link
- [[ownership|Section @fold 6]] Same-file heading link

![[kb]] Embed full file
![[kb#Heading]] Embed section
![[image.png]] Embed image
![[image.png|300]] Embed image with width

[[share-manifest-assets]]

## Decorations

- [Capabilities](https://code.visualstudio.com/api/extension-capabilities/overview)
- [Decoration Render Options](https://code.visualstudio.com/api/references/vscode-api#DecorationRenderOptions)

Types

- [x] Text "before" the line/code.
- [x] Line "hightlight"
- [x] Text "after" the line/code.
- [x] Icon above...? Can't tell
- [ ] Squiggly lines.
- [ ] Link above text with click behavior.
- [ ] Icon in gutter
- [ ] Hover Text/Icon reveals panel

### Heading 3

#### Heading 4

##### Heading 5

###### Heading 6

## Local Development

**Launch.json**

```sh
f5
```

**create build, vsix**

```sh
git clone git@github.com:PrimeTimeTran/markdown-wiki-links.git
cd markdown-wiki-links
pnpm run build
pnpm add -D vscode
pnpm build
pnpm add -g @vscode/vsce
vsce --version
pnpm package
markdown-wiki-links-0.2.1.vsix
```

- Builds local pkg
- Used to install in production VSCode
- Used to release

## Local Development

- https://code.visualstudio.com/api/references/vscode-api#window.createTextEditorDecorationType
- https://code.visualstudio.com/api/references/vscode-api#TextEditor
- https://code.visualstudio.com/api/references/vscode-api#TextEditorDecorationType
- [github](https://github.com/PrimeTimeTran)
- [Google](https://www.google.com)
- [Wikipedia](https://en.wikipedia.org/wiki/Quantopian)
- [[ownership|Span]]
