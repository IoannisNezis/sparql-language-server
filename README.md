# Fichu

A blazingly fast formatter and [language server](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification) for [SPARQL](https://de.wikipedia.org/wiki/SPARQL), written in Rust :crab:.

# Getting Started

## Installation

Fichu is availible on [PyPi](https://pypi.org/project/fichu/):

```shell
pipx install fichu
```

Or on [crate.io](https://crates.io/crates/fichu):

```shell
cargo install fichu
```

You can also build it from source:


```shell
git clone https://github.com/IoannisNezis/sparql-language-server.git
cd sparql-language-server
cargo build --release
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
