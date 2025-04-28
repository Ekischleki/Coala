# Coala
Coloring a graph is a fun activity for the entirety of your family to enjoy. The rules are simple: Color each node of a graph with the least possible amount of colors so that no connected nodes share a color. In our case, you must color the graph with exactly 3 colors. But how can that result in any complex behaviour? It's simple! You start with two logic gates: A not gate which when you give color a as input outputs color b on its output node, and an or gate which outputs a if either or both of the inputs are of color a.
Chaining together these two cute logic gates, allows for any possible truth table, multiplying integers, playing Super Mario by emulating a cpu, you name it.
Building such a graph yourself however seems to be quite complex, which is where the coala compiler comes into play. It also starts with the two fundemental building blocks, a not and an or gate, but quickly abstracts them away using a human readable programming language which allows functions and structures and whatnot. Having such a strange environment to compile to of course leaves the language which some restrictions compared to modern computers, which we'll talk about later.
Im sure by now you're already excited to learn this language and explore the countless possibilities this new environment presents to you, so let's start by exploring some basics.

## Compiler usage
Until now, the compiler doesn't support many arguments. You just simply use it as such: `./coala source_file`. If you run that, it might output a whole bunch of debug stuff, but you're safe to just ignore that. The two formatted files it outputs are compiled_edges.csv and compiled_labels.csv, representing the graph and the coloring respectively. You can visualize these with a tool like gephi.
## Comments
Perhaps the most important thing to start out with, so code can actually be explained: Comments. Everything past a `#` sign gets turned into a comment up until a new line is reached
```
#This is a comment
This is no longer a comment
#This is another comment
#Amazing
```
## Atom types
Atom types are the simple True and False values you have access to. Everything you make in this language will at its lowest level be made from the two atom types there are, similar to how a computer is "just zeros and ones" at its lowest level. 
## Atom subs
Atom subs (Short for atomic submarines/substructures however you're feeling) are the two subs you have access to from the beginning of every file. They're compiler built-in. They are meant to combine atom types.
The two atom subs are "not" and "or". Here are their respective truth tables
- not
  - not (True) => False
  - not (False) => True
- or
  - or (False, False) => False
  - or (False, True) => True
  - or (True, False) => True
  - or (True, True) => True

But im sure you know these already ;) 
## Subs (functions) and collections
By chaining together the two atom subs you were given, you can make every single truth table you can imagine, by creating your own subs.
First think of the general topic you want your subs to be in, like commutative two input truth tables. The group name you have in mind is gonna be your collection, in which you can put subs with their own name. Here's an example
```
collection commutative_two_input_truth_tables {
  #Youll figure out a bit more about types in the types section.
  sub and(bool: a, bool: b) {
    let res = #Our code would go here, if we know how to write any...
  } = res #Ohh yeah. Subs can only return something after everything has been executed. This is shown by the assignage. For this assignage, you'll have access to all the variables accessable within the function.
}
```
## Problem solution architecture
Every problem in this language represents a starting point of the program. There can be multiple problems, which would corrospond to multiple starting points. The execution order is just kinda "whatever I feel like rn" with this language so don't worry about it. Every problem may take some inputs, specifying these inputs is what the solution is for. If the given inputs are "correct", the graph can easily be colored by the compiler. Here's an example of how you would use these
```
problem {
  sub main_problem(bool: a, bool: b) {
    #Make it so that the only correct solution to the problem is when both a and b are false, youll learn about this later
    force or(a, b) => false
  }
}

solution {
  main_problem(false, false), #Add more solutions by comma seperating them.
}
```

## Composite types
Composite types are basically a collection of named values. You might know them as structs. Here's how you define a composite type
```
composite CoolType {
  bool: is_cool,
  
}
```
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



