# Kanpai 🥂

A demo of a logic programming language for inferring types,
based on [miniKanren](http://minikanren.org/).

## Usage

In order to run a Kanpai file:

```sh
$ git clone git@github.com:JulianKnodt/kanpai.git
$ cd kanpai
$ cargo run -- --file <filename>.kanpai
```

## What is Logic Programming?

Logic Programming is a Domain Specific Language (DSL), meant to describe relationships between
things, so that if we know something, we may be able to infer other properties.
As a classic example of logic, consider this:
```
Aristotle is human
All humans are mortal.
Therefore, Aristotle is mortal.
```

If we were to describe this in a programming language, we might say something like:
```ocaml
let aristotle : "Human" in
for all : "Human" -> "Mortal"
thus aristotle : "Mortal"
```

Thus, logic programming seeks to take deductive reasoning that people use trying to understand
something, and encompass it in a programming language for ease of use. The two most famous forms
of logic programming languages are [Prolog](https://en.wikipedia.org/wiki/Prolog), and
[miniKanren](http://minikanren.org/), but they are not often used, because, well, it's not clear
what you would use them for. If you want to deduce something, it's probably faster to deduce it
yourself, and if you want to prove something, well that's also generally easier to write by
hand. Now, if you want to prove many things, maybe it makes sense to automate the process, but
it's not clear where you would need to have many things you would want to prove.

Well, it turns out programmers are actually implicitly proving things about their programs all
the time. Specifically, that they type check!

### What is type checking?

Type checking is a step that a compiler or interpreter does to ensure that a program can
correctly execute. For example, it doesn't make sense to perform the operation `"wat" + []`,
[obviously](https://www.destroyallsoftware.com/talks/wat). We would like to know that at compile
time, and report it back to the user. Type checking is a feature of
[statically-typed](https://developer.mozilla.org/en-US/docs/Glossary/Static_typing) programming
languages, and what a type checker does depends on the complexity of the type system and the
specific rules each type system has.

Type checking is usually done during compilation, after parsing the syntax of a program, and
before emitting the actual instructions that get executed on a machine, and since it is needed
every time a program is compiled, it must be run quickly.

### Combining Logic Programming with a Type Checker.

To make it a bit more clear how type checking is a "proof", let's consider a C program.
```c
int main(void) {
  int a = 3;
  int b;
  b = a;
  a = "example"; // * doesn't type check

  return 0;
}
```

In order for this to compile, we have to know how much space each variable takes, and in order
to know how much space each variable takes, we have to know its type. Thus, we want to ensure
that each value only takes valid values. In order to show this, we check that each statement and
expression is valid, and if they are all valid, then the entire function type checks, and if all
functions type check, then the entire program type checks. Type checking is thus defined
recursively, for each statement. To show that each statement (i.e. `int a = 3;`) type checks,
currently each compiler will probably define its own approach, and there has been no unified
approach for handling type checking that can be reused across compilers. Why is this the case?
Mostly because type systems across different languages are different. Some languages may
have more complex type systems, (i.e. C++ with templates and multiple-inheritance), or very
simple type systems (i.e. C with basically no generics). Thus, what might be necessary for one
language might be unnecessary or insufficient for another language.

Thus far, I've been begging the question, is there a way to unify type checking across
languages? And to that, I _think_ the answer is yes, using logic programming (otherwise why
would I be writing this?). We can express the above program as this in logic:
```ocaml
let a: 3 in
let b: Number in
constrain a = b in
possible a
constrain a = "example" in
possible a
```
What "possible" does is output the possible type of a variable, with a few key types:
`* (Dynamic)` which is any possible type, `! (Never)` which is no possible types,
and `Param(x)` which refers to another type. If a variable is `!`, the program will not type
check. In the example above, the second `possible a` will print `a : !`, indicating that the
program does not type check.

## Uses for Kanpai

Kanpai implements a basic type checker for a made-up type system, which includes tuples,
numbers, strings, and booleans, and a strange version of enums.

Since this is just a toy language, it's usefulness is limited, and it may have bugs.
It may be usable at least as a typechecker for parts of C, and has features such as tuples
and enums. There are some sample type checking programs inside of `/samples`.

To give an example you can express constraints between complex types through other types:

```ocaml
let x: Text in
let y: (x, 5) in
let z: ("test", *) in
constrain y = z in
possible y
possible x
thus x: "test"
```

There are still a few bugs where execution order will affect equality, but I currently can't be
bothered to hunt down these bugs, because I think it'd be more productive to rewrite the whole
language implementation.

I think what is missing is the ability to define new rules on types, but again I think that
would be suited for a different language.

### Prolog for type checking?

It doesn't make sense to use this toy language for type checking, but using a thorough
prolog implementation such as [this](https://github.com/mthom/scryer-prolog) might make sense.
The key thing is that it must execute quickly, and its not immediately clear to me if it would
always be, since I'm not sure if uses memory efficiently, and if it does only as much work as
necessary. That being said, it's probably the closest thing currently.

#### Why the name "Kanpai"?

Kanpai was originally based on miniKanren, so I yoinked the first three letters (which is the
first character) to keep it similar looking. Since this is a toy language, I called it
Kanpai, which is a toast since this is meant to be a fun language. I also modified
the syntax to be readable by a normal human, since I think existing logic programming languages
are a pain in the ass to read if you're not familiar with the syntax.

Syntax highlighting is also done as if it were OCaml, since I borrowed some of the syntax from
that and Rust.



