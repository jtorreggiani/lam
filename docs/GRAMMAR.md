# Proto Grammar

This grammar is for a simple subset of Prolog that the LAM could be used to evaluate. It is not complete an just an example.

```ebnf
(* A Prolog program is a sequence of clauses. *)

```

(_ A complete Prolog program is a sequence of clauses. _)
Program ::= { Clause "." } .

(_ A clause is either a fact or a rule. _)
Clause ::= Atom [ ":-" Body ] .

(_ The head of a clause is an atom; the body is a (possibly empty) comma‐separated list of atoms. _)
Body ::= Atom { "," Atom } .

(_ Atoms are typically predicates with arguments (which are terms). _)
Atom ::= Predicate "(" TermList ")"
| Term

(_ A comma‐separated list of terms. _)
TermList ::= Term { "," Term } .

(_ Terms include constants, variables, compound terms, lambda abstractions, and applications. _)
Term ::= Constant
| Variable
| Compound
| Lambda
| Application

(_ A constant is an integer literal. _)
Constant ::= Digit { Digit } .
Digit ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"

(_ A variable is an identifier beginning with an uppercase letter or underscore. _)
Variable ::= ( Uppercase | "_" ) { Letter | Digit | "_" } .
Uppercase ::= "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J"
| "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T"
| "U" | "V" | "W" | "X" | "Y" | "Z"
Letter ::= Uppercase | Lowercase
Lowercase ::= "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j"
| "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t"
| "u" | "v" | "w" | "x" | "y" | "z"

(_ A compound term is a functor with arguments. _)
Compound ::= Functor "(" TermList ")" .
Functor ::= Lowercase { Letter | Digit | "\_" } .

(_ A lambda abstraction is written with the keyword "lambda" (or a lambda symbol)
followed by a variable, a dot, and a body term. _)
Lambda ::= "lambda" Variable "." Term
| "λ" Variable "." Term

(_ An application is represented as juxtaposition of two terms.
(We assume left-associativity, so that f x y is parsed as ((f x) y).)
Parentheses may be used for clarity. _)
Application ::= Term Term
| "(" Term ")" .

```

```
