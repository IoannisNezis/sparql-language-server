> [!CAUTION]
> This Project is still in an early stage.  
> Only the format capability is production ready.  
> The rest is experimental.

<h1 align="center">
  Fichu 🦀
</h1>

⚡A blazingly fast [language server](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification) for [SPARQL](https://de.wikipedia.org/wiki/SPARQL), written in Rust 🦀.

# Getting Started

## Installation

Fichu is available on [crate.io](https://crates.io/crates/fichu):

```shell
cargo install fichu
```

And on [PyPi](https://pypi.org/project/fichu/):

```shell
pipx install fichu
```

You can also build it from source:

```shell
git clone https://github.com/IoannisNezis/sparql-language-server.git
cd sparql-language-server
cargo build --release --bin fichu
```

## Usage

To run Fichu as **formatter** run:

```shell
fichu format <PATH>
```

To run Fichu as **lanugage server** run:

```shell
fichu server
```

This will create a language server listening on stdio.

## Connect to Neovim

After you installed the language server, add this to your `init.lua`:

```lua
vim.api.nvim_create_autocmd({ 'FileType' }, {
  desc = 'Connect to sparql-language-server',
  pattern = { 'sparql' },
  callback = function()
    vim.lsp.start {
      name = 'fichu',
      cmd = { 'fichu', 'server' },
      root_dir = vim.fn.getcwd(),
      on_attach = function(client, bufnr)
        vim.keymap.set('n', '<leader>f', vim.lsp.buf.format, { buffer = bufnr, desc = 'LSP: ' .. '[F]ormat' })
      end,
    }
  end,
})
```

Open a `.rq` file and check that the buffer is attached to th server:

```
:checkhealth lsp
```

Configure keymaps in `on_attach` function.

# Capabilities

## Formatting

**Status**: Full support

Formats SPARQL queries to ensure consistent and readable syntax.
Customizable options to align with preferred query styles are also implemented.

## Diagnostics

**Status**: Partial support

Currently provides a few basic diagnostics for syntax errors and simple issues in SPARQL queries.
Further enhancements are planned to cover a broader range of semantic and logic-related diagnostics.

**Currently provided diagnostics**:

- unused namespace (warning): A declared namespace is not used
- undefined namespace (error): A used namespace is not declared

**Planed diagnostics**:

- path compresion possible (info): A declared namespace is not used

## Completion

**Status**: Rudimentary

Basic auto-completion for SPARQL keywords and variables. Currently not context aware.  
Future improvements will expand suggestions to include functions, predicates, and custom completions based on query context.

## Code Actions

**Status**: Planed

Future support for code actions, such as quick fixes and refactoring suggestions, to improve productivity and code quality in SPARQL development.

**Planed code actions**:

- Consolidate property paths
- Refactor iris into namespaces
- Sort Prefixes

# Configuration

Fichu can be configured through a `fichu.toml` or `fichu.yml` file.

Here is the full default configuration
```toml
[format]
align_predicates = false
align_prefixes = false
separate_prolouge = true
capitalize_keywords = true
insert_spaces = true
tab_size = 2
where_new_line = true
```

# use in web

If you want to connect from a web-based-editor, you can use this package as well.  
For this purpouse this can be compiled to wasm and is availible on [npm](https://www.npmjs.com/package/@ioannisnezis/sparql-language-server):


```shell
npm i @ioannisnezis/sparql-language-server
```

You will have to wrap this in a Web Worker and provide a language server client.
There will be more documentation on this in the future...

## Demo

In the mean time, check out the [demo](https://sparql.nezis.de).
