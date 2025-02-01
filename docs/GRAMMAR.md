```
(* A complete Prolog program is a sequence of clauses. *)
Program       ::= { Clause "." } .

(* A clause is either a fact or a rule. *)
Clause        ::= Atom [ ":-" Body ] .

(* The head of a clause is an atom; the body is a (possibly empty) comma‐separated list of atoms. *)
Body          ::= Atom { "," Atom } .

(* Atoms are typically predicates with arguments (which are terms). *)
Atom          ::= Predicate "(" TermList ")"
                | Term

(* A comma‐separated list of terms. *)
TermList      ::= Term { "," Term } .

(* Terms include constants, variables, compound terms, lambda abstractions, and applications. *)
Term          ::= Constant
                | Variable
                | Compound
                | Lambda
                | Application

(* A constant is an integer literal. *)
Constant      ::= Digit { Digit } .
Digit         ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"

(* A variable is an identifier beginning with an uppercase letter or underscore. *)
Variable      ::= ( Uppercase | "_" ) { Letter | Digit | "_" } .
Uppercase     ::= "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J"
                | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T"
                | "U" | "V" | "W" | "X" | "Y" | "Z"
Letter        ::= Uppercase | Lowercase
Lowercase     ::= "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j"
                | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t"
                | "u" | "v" | "w" | "x" | "y" | "z"

(* A compound term is a functor with arguments. *)
Compound      ::= Functor "(" TermList ")" .
Functor       ::= Lowercase { Letter | Digit | "_" } .

(* A lambda abstraction is written with the keyword "lambda" (or a lambda symbol)
   followed by a variable, a dot, and a body term. *)
Lambda        ::= "lambda" Variable "." Term
                | "λ" Variable "." Term

(* An application is represented as juxtaposition of two terms.
   (We assume left-associativity, so that f x y is parsed as ((f x) y).)
   Parentheses may be used for clarity. *)
Application   ::= Term Term
                | "(" Term ")" .
```
