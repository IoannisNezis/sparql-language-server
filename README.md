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
