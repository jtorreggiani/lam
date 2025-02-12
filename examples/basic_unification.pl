parent(john, mary).

main :-
  parent(john, X),
  write(X),
  nl,
  halt.