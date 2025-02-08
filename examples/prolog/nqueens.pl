% nqueens_benchmark.pl
%
% This SWI Prolog script defines the classic N-Queens problem using a permutation-based
% approach and benchmarks its execution. In addition, it converts one solution
% (for N=4) into a custom cons-list representation:
%
%   cons(2, cons(4, cons(1, cons(3, 0))))
%
% where 0 represents nil.

% --- N-Queens Predicates ---

% nqueens(+N, -Qs)
% Qs is a permutation of [1,2,...,N] such that no two queens attack each other.
nqueens(N, Qs) :-
    numlist(1, N, Domain),
    permutation(Domain, Qs),
    safe(Qs).

% safe(+Qs)
% Succeeds if the list of queen positions Qs is safe (no diagonal attacks).
safe([]).
safe([Q|Qs]) :-
    safe_aux(Q, Qs, 1),
    safe(Qs).

% safe_aux(+Q, +Qs, +D)
% For a queen Q and the rest Qs, with diagonal offset D,
% ensures that for every queen Q1 in Qs, abs(Q - Q1) =\= D.
safe_aux(_, [], _).
safe_aux(Q, [Q1|Qs], D) :-
    abs(Q - Q1) =\= D,
    D1 is D + 1,
    safe_aux(Q, Qs, D1).

% --- Conversion Predicate ---
%
% to_cons(+List, -Cons)
% Converts a standard Prolog list into a cons–list representation
% where the empty list is represented as the constant 0.
to_cons([], 0).
to_cons([H|T], cons(H, CT)) :-
    to_cons(T, CT).

% --- Main Predicate for Benchmarking ---
main :-
    % Set board size N; here we choose 4 for demonstration.
    N = 4,
    format("Running N-Queens for N = ~w ...~n", [N]),
    statistics(runtime, [Start|_]),
    % Find all solutions.
    findall(Qs, nqueens(N, Qs), Solutions),
    statistics(runtime, [End|_]),
    Time is End - Start,
    length(Solutions, Count),
    format("Found ~w solutions for ~w-Queens in ~w ms.~n", [Count, N, Time]),
    % For demonstration, convert the first solution to the cons representation.
    Solutions = [FirstSolution|_],
    to_cons(FirstSolution, ConsSolution),
    format("One solution in cons representation: ~w~n", [ConsSolution]),
    halt.

:- initialization(main, main).
