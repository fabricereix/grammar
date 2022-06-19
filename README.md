# Grammar
Validate and generate HTML Grammar.

The grammar is a BNF-like grammar,
similar to the one used in [Kotlin](https://kotlinlang.org/docs/reference/grammar.html) and [Rust](https://doc.rust-lang.org/stable/reference/expressions.html).

It is described in its own [grammar](https://htmlpreview.github.io/?https://raw.githubusercontent.com/fabricereix/grammar/main/samples/grammar.html)

## Example 

U.S. postal address from wikipedia (https://en.wikipedia.org/wiki/Backus%E2%80%93Naur_form)

```
# Postal Address

postal-address: name-part street-address zip-part

name-part: personal-part last-name opt-suffix-part | personal-part name-part

personal-part: initial "." | first-name

street-address: house-num street-name opt-apt-num

zip-part: town-name "," state-code ZIP-code

opt-suffix-part: "Sr." | "Jr." | ""

opt-apt-num: apt-num | ""

...
```

Here is the generated [HTML](https://htmlpreview.github.io/?https://raw.githubusercontent.com/fabricereix/grammar/main/samples/address.html)



## Usage

```
grammar FILE
Validate grammar file and generate its HTML representation.
```

Validation rules:

- every non-terminal is defined
- every rule is used (except the first one)

## Installation

Precompiled binaries are available for Linux and MacOS in https://github.com/fabricereix/grammar/releases.

```
$ INSTALL_DIR=/tmp
$ curl -sL https://github.com/fabricereix/grammar/releases/download/0.1.0/grammar-0.1.0-x86_64-linux.tar.gz | tar xvz -C $INSTALL_DIR
$ export PATH=$INSTALL_DIR/grammar-0.1.0:$PATH
```





