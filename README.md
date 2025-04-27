# Coala
Coloring a graph is a fun activity for the entirety of your family to enjoy. The rules are simple: Color each node of a graph with the least possible amount of colors so that no connected nodes share a color. In our case, you must color the graph with exactly 3 colors. But how can that result in any complex behaviour? It's simple! You start with two logic gates: A not gate which when you give color a as input outputs color b on its output node, and an or gate which outputs a if either or both of the inputs are of color a.
Chaining together these two cute logic gates, allows for any possible truth table, multiplying integers, playing Super Mario by emulating a cpu, you name it.
Building such a graph yourself however seems to be quite complex, which is where the coala compiler comes into play. It also starts with the two fundemental building blocks, a not and an or gate, but quickly abstracts them away using a human readable programming language which allows functions and structures and whatnot. Having such a strange environment to compile to of course leaves the language which some restrictions compared to modern computers, which we'll talk about later.
Im sure by now you're already excited to learn this language and explore the countless possibilities this new environment presents to you, so let's start by exploring some basics.

## Compiler usage

## Comments
Perhaps the most important thing to start out with, so code can actually be explained: Comments. Everything past a `#` sign gets turned into a comment up until a new line is reached
```
#This is a comment
This is no longer a comment
#This is another comment
#Amazing
```
## Atom types

## Atom subs

## Subs (functions) and collections

## Problem solution architecture
## Problem
## Solution

## Composite types

## Type syntax

## Expressions
Expressions simply transform some input values into a now output value.
### Variable expression
To access a variable, simply type out the variable's name wherever an expression is expected.
### Literal expressions
To access the value of an atom type, simply type out its name. It is generally discouraged because having a literal atom type in your code almost always means that expressions can be simplified the compiler will do that automatically, so don't worry).
### Tuple expressions
A tuple is an ordered collection of unnamed values. They are pretty much the same as you might know them in other languages. To create a tuple simply type out the expressions you want in it as a comma seperated list enclosed by parenthasees.
Pretty similar to most other languages. Here's an example:
```
#This is a tuple expression
(variable_one, false, (), ((true), false))
#The tuple contains first a variable, then a literal false value, then an empty tuple, then a tuple with another tuple inside and a literal respectively.
```
Empty tuples represent what is `void` in most other languages. All subs which don't explicitly return anything, return an empty tuple.
### Sub call expressions
Calling a sub is an expression, which leaves the _result_ of the sub as a value in its place. You use it by first typing out the sub's collection name then a double colon then the sub's name and then an expressions with its input values.
```
#We have a collection bool, with functions 'and' and assert_true inside
bool::and(a, b)
#In this case (a, b) is a single tuple expression of two values. It leaves in place whatever is defined as the result of this operation

bool::assert_true(false)
#In this case (false) is a tuple with one element inside, at which point you can also remove the tuple

bool::assert_true false
#This amounts to the same as the previous example.
```

### Composite constructor expressions
### Access expressions
### Access index expressions
## Code statements
### Let
### Force
### Sub call statement
### If
### Ifelse



