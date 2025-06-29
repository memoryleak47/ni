Ni
==

This research project is trying out a new point in the design space of python analysis.
The idea is to compile the rich python language down to a simpler language, by making all the implicit things explicit.
We hope that this leads us to a language, whose individual building blocks have a much easier semantics, and are hence easier to analyze.
We aim for a linear compilation size, i.e. each input statement should only grow by a constant factor during this translation.

We intend to break down
- recursion, iteration (loops), generators and exceptions down to a single control-flow primitive (namely "procs", which are basically addresses you can jump to), and
- compound objects like closures or class-instances into separate smaller pieces (like fn-ptrs, a dict, a runtime-typeinfo-tag etc.) which individually have an easier semantics, and
- concise python syntax down to more verbose but simpler instructions (i.e. resolve operator overloads, list comprehensions, add explicit code for type coercion, make truthiness casting in conditions explicit etc.)
