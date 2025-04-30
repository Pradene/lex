# Lex

This project is a lexer generator from a syntax file written in rust.  
Generate C code from a syntax file (See examples).   
(Currently in development).

## Usage

1. Generate and compuile the lexer code with make:
``` bash
make
```

2. Run the program:
``` bash
./lex
```

You can change the syntax file inside the Makefile

## Ressources

[NFA](https://en.wikipedia.org/wiki/Nondeterministic_finite_automaton)  
[DFA](https://en.wikipedia.org/wiki/Deterministic_finite_automaton)  
[Thompson's construction](https://en.wikipedia.org/wiki/Thompson%27s_construction)  
[Guide to Lex and Yacc](https://arcb.csc.ncsu.edu/~mueller/codeopt/codeopt00/y_man.pdf)  
[Regular Expression Matching using Bit Vector Automata](https://ohyoukillkenny.github.io/source/BVA.pdf)  
[NFA vs DFA](https://www.abstractsyntaxseed.com/blog/regex-engine/nfa-vs-dfa)  
[Compiler introduction](https://www.cs.cornell.edu/courses/cs4120/2023sp/notes.html?id=leximpl#:~:text=A%20lexer%20generator%20works%20by,for%20a%20table%2Ddriven%20lexer.)  
[Offset-FA](https://pmc.ncbi.nlm.nih.gov/articles/PMC9607373/)  
