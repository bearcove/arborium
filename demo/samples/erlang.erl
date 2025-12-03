%%
%% %CopyrightBegin%
%%
%% SPDX-License-Identifier: Apache-2.0
%%
%% Copyright Ericsson AB 1996-2025. All Rights Reserved.
%%
%% Licensed under the Apache License, Version 2.0 (the "License");
%% you may not use this file except in compliance with the License.
%% You may obtain a copy of the License at
%%
%%     http://www.apache.org/licenses/LICENSE-2.0
%%
%% Unless required by applicable law or agreed to in writing, software
%% distributed under the License is distributed on an "AS IS" BASIS,
%% WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
%% See the License for the specific language governing permissions and
%% limitations under the License.
%%
%% %CopyrightEnd%
%%
-module(lists).
-moduledoc """
List processing functions.

This module contains functions for list processing.

Unless otherwise stated, all functions assume that position numbering starts
at 1. That is, the first element of a list is at position 1.

Two terms `T1` and `T2` compare equal if `T1 == T2` evaluates to `true`. They
match if `T1 =:= T2` evaluates to `true`.

Whenever an _ordering function_{: #ordering_function } `F` is expected as
argument, it is assumed that the following properties hold of `F` for all x, y,
and z:

- If x `F` y and y `F` x, then x = y (`F` is antisymmetric).
- If x `F` y and y `F` z, then x `F` z (`F` is transitive).
- x `F` y or y `F` x (`F` is total).

An example of a typical ordering function is less than or equal to: `=</2`.
""".

-compile({no_auto_import,[max/2]}).
-compile({no_auto_import,[min/2]}).

%% BIFs (implemented in the runtime system).
-export([keyfind/3, keymember/3, keysearch/3, member/2, reverse/2]).

%% Miscellaneous list functions that don't take funs as
%% arguments. Please keep in alphabetical order.
-export([append/1, append/2, concat/1,
         delete/2, droplast/1, duplicate/2,
         enumerate/1, enumerate/2, enumerate/3,
         flatlength/1, flatten/1, flatten/2,
         join/2, last/1, min/1, max/1,
         nth/2, nthtail/2,
         prefix/2, reverse/1, seq/2, seq/3,
         split/2, sublist/2, sublist/3,
         subtract/2, suffix/2, sum/1,
         uniq/1, unzip/1, unzip3/1,
         zip/2, zip/3, zip3/3, zip3/4]).

%% Functions taking a list of tuples and a position within the tuple.
-export([keydelete/3, keyreplace/4, keymap/3,
         keytake/3, keystore/4]).

%% Sort functions that operate on list of tuples.
-export([keymerge/3, keysort/2, ukeymerge/3, ukeysort/2]).

%% Sort and merge functions.
-export([merge/1, merge/2, merge/3, merge3/3,
         sort/1, sort/2,
         umerge/1, umerge/2, umerge/3, umerge3/3,
         usort/1, usort/2]).

%% Functions that take fun arguments (high-order functions). Please
%% keep in alphabetical order.
-export([all/2, any/2, dropwhile/2,
         filter/2, filtermap/2, flatmap/2,
         foldl/3, foldr/3, foreach/2,
         map/2, mapfoldl/3, mapfoldr/3,
         partition/2, search/2,
         splitwith/2, takewhile/2, uniq/2,
         zipwith/3, zipwith/4, zipwith3/4, zipwith3/5]).

%% Undocumented old name for filtermap
-export([zf/2]).
-deprecated([{zf,2,"use filtermap/2 instead"}]).

%% Undocumented and unused merge functions for lists sorted in reverse
%% order. They are exported so that the fundamental building blocks
%% for the sort functions can be tested. (Removing them would save
%% very little because they are thin wrappers calling helper functions
%% used by the documented sort functions.)
-export([rkeymerge/3, rmerge/2, rmerge/3, rmerge3/3,
         rukeymerge/3, rumerge/2, rumerge/3, rumerge3/3]).

%% Shadowed by erl_bif_types: lists:keyfind/3
-doc """
Searches the list of tuples `TupleList` for a tuple whose `N`th element compares
equal to `Key`.

Returns `Tuple` if such a tuple is found; otherwise, returns `false`.

## Examples

```erlang
1> lists:keyfind(b, 1, [{a,10}, {b,20}, {c,30}]).
{b,20}
2> lists:keyfind(unknown, 1, [{a,10}, {b,20}, {c,30}]).
false
```
""".
-spec keyfind(Key, N, TupleList) -> Tuple | false when
      Key :: term(),
      N :: pos_integer(),
      TupleList :: [Tuple],
      Tuple :: tuple().
