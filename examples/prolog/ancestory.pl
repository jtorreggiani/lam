% Facts about parents
parent(john, mary).  
parent(john, tom).    
parent(jane, mary).    
parent(jane, tom).    
parent(mary, ann).     
parent(tom, peter).    

% Recursive rule: ancestor
% A person X is an ancestor of Y if X is a parent of Y,
% or if X is a parent of some Z who is an ancestor of Y.
ancestor(X, Y) :- parent(X, Y).
ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y).

% Simple display: print all ancestor relationships.
main :-
    write('Ancestor relationships:'), nl,
    ancestor(john, X),
    write('- '),
    write(X),
    write('\n'),
    fail.
main.

:- main, halt.