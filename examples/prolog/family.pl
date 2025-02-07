% Facts about parents
parent(john, mary).  
parent(john, tom).    
parent(jane, mary).    
parent(jane, tom).    
parent(mary, ann).     
parent(tom, peter).    

% Facts about gender
female(jane).
female(mary).
female(ann).
male(john).
male(tom).
male(peter).

% Rule
mother(X, Y) :- parent(X, Y), female(X).

% Simple display
main :- mother(X, Y), write(X-Y), nl, fail.
main.

:- main, halt.