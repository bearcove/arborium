module Data.List

import public Control.Function

import Data.Nat
import Data.List1
import Data.Fin
import public Data.Zippable

%default total

||| Boolean check for whether the list is the empty list.
public export
isNil : List a -> Bool
isNil [] = True
isNil _  = False

||| Boolean check for whether the list contains a cons (::) / is non-empty.
public export
isCons : List a -> Bool
isCons [] = False
isCons _  = True

||| Add an element to the end of a list.
||| O(n). See the `Data.SnocList` module if you need to perform `snoc` often.
public export
snoc : List a -> a -> List a
snoc xs x = xs ++ [x]

||| Take `n` first elements from `xs`, returning the whole list if
||| `n` >= length `xs`.
|||
||| @ n  the number of elements to take
||| @ xs the list to take the elements from
public export
take : (n : Nat) -> (xs : List a) -> List a
take (S k) (x :: xs) = x :: take k xs
take _ _ = []

||| Remove `n` first elements from `xs`, returning the empty list if
||| `n >= length xs`
|||
||| @ n  the number of elements to remove
||| @ xs the list to drop the elements from
public export
drop : (n : Nat) -> (xs : List a) -> List a
drop Z     xs      = xs
drop (S n) []      = []
drop (S n) (_::xs) = drop n xs

||| Satisfiable if `k` is a valid index into `xs`.
|||
||| @ k  the potential index
||| @ xs the list into which k may be an index
public export
data InBounds : (k : Nat) -> (xs : List a) -> Type where
    ||| Z is a valid index into any cons cell
    InFirst : InBounds Z (x :: xs)
    ||| Valid indices can be extended
    InLater : InBounds k xs -> InBounds (S k) (x :: xs)

public export
Uninhabited (InBounds k []) where
  uninhabited InFirst impossible
  uninhabited (InLater _) impossible

export
Uninhabited (InBounds k xs) => Uninhabited (InBounds (S k) (x::xs)) where
  uninhabited (InLater y) = uninhabited y

||| Decide whether `k` is a valid index into `xs`.
public export
inBounds : (k : Nat) -> (xs : List a) -> Dec (InBounds k xs)
inBounds _ [] = No uninhabited
inBounds Z (_ :: _) = Yes InFirst
inBounds (S k) (x :: xs) with (inBounds k xs)
  inBounds (S k) (x :: xs) | (Yes prf) = Yes (InLater prf)
  inBounds (S k) (x :: xs) | (No contra)
      = No $ \(InLater y) => contra y

||| Find a particular element of a list.
|||
||| @ ok a proof that the index is within bounds
public export
index : (n : Nat) -> (xs : List a) -> {auto 0 ok : InBounds n xs} -> a
index Z (x :: xs) {ok = InFirst} = x
index (S k) (_ :: xs) {ok = InLater _} = index k xs

public export
index' : (xs : List a) -> Fin (length xs) -> a
index' (x::_)  FZ     = x
index' (_::xs) (FS i) = index' xs i

||| Generate a list by repeatedly applying a partial function until exhausted.
||| @ f the function to iterate
||| @ x the initial value that will be the head of the list
covering
public export
iterate : (f : a -> Maybe a) -> (x : a) -> List a
iterate f x  = x :: case f x of
  Nothing => []
  Just y => iterate f y

||| Given a function `f` which extracts an element of `b` and `b`'s
||| continuation, return the list consisting of the extracted elements.
||| CAUTION: Only terminates if `f` eventually returns `Nothing`.
|||
||| @ f  a function which provides an element of `b` and the rest of `b`
||| @ b  a structure containing any number of elements
covering
public export
unfoldr : (f : b -> Maybe (a, b)) -> b -> List a
unfoldr f c = case f c of
  Nothing     => []
  Just (a, n) => a :: unfoldr f n

||| Returns the list of elements obtained by applying `f` to `x` `0` to `n-1` times,
||| starting with `x`.
|||
||| @ n  the number of times to iterate `f` over `x`
