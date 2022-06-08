# Kanpai 🥂

A demo of a logic programming language for inferring types,
based on [miniKanren](http://minikanren.org/).

## Logic Programming

Logic Programming is a Domain Specific Language, intended for describing relationships between
objects, so that if we know something about an object, we may be able to infer other properties.
As a classic example, consider these statements:
```
Aristotle is human
All humans are mortal.
Therefore, Aristotle is mortal.
```

If we were to describe this in a programming language, we might say something like:
```
let aristotle : "Human" in
for all "Human" -> "Mortal"
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

