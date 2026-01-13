------------------------------------------------------------------------
-- The Agda standard library
--
-- Lists, basic types and operations
------------------------------------------------------------------------

-- See README.Data.List for examples of how to use and reason about
-- lists.

{-# OPTIONS --cubical-compatible --safe #-}

module Data.List.Base where

open import Algebra.Bundles.Raw using (RawMagma; RawMonoid)
open import Data.Bool.Base as Bool
  using (Bool; false; true; not; _∧_; _∨_; if_then_else_)
open import Data.Fin.Base using (Fin; zero; suc)
open import Data.Maybe.Base as Maybe using (Maybe; nothing; just; maybe′)
open import Data.Nat.Base as ℕ using (ℕ; zero; suc)
open import Data.Product.Base as Product using (_×_; _,_; map₁; map₂′)
open import Data.Sum.Base as Sum using (_⊎_; inj₁; inj₂)
open import Data.These.Base as These using (These; this; that; these)
open import Function.Base
  using (id; _∘_ ; _∘′_; _∘₂_; _$_; const; flip)
open import Level using (Level)
open import Relation.Unary using (Pred; Decidable)
open import Relation.Binary.Core using (Rel)
import Relation.Binary.Definitions as B
open import Relation.Binary.PropositionalEquality.Core using (_≡_)
open import Relation.Nullary.Decidable.Core using (T?; does; ¬?)

private
  variable
    a b c p ℓ : Level
    A : Set a
    B : Set b
    C : Set c

------------------------------------------------------------------------
-- Types

open import Agda.Builtin.List public
  using (List; []; _∷_)

------------------------------------------------------------------------
-- Operations for transforming lists

map : (A → B) → List A → List B
map f []       = []
map f (x ∷ xs) = f x ∷ map f xs

infixr 5 _++_

_++_ : List A → List A → List A
[]       ++ ys = ys
(x ∷ xs) ++ ys = x ∷ (xs ++ ys)

intersperse : A → List A → List A
intersperse x []       = []
intersperse x (y ∷ []) = y ∷ []
intersperse x (y ∷ ys) = y ∷ x ∷ intersperse x ys

intercalate : List A → List (List A) → List A
intercalate xs []         = []
intercalate xs (ys ∷ [])  = ys
intercalate xs (ys ∷ yss) = ys ++ xs ++ intercalate xs yss

cartesianProductWith : (A → B → C) → List A → List B → List C
cartesianProductWith f []       _  = []
cartesianProductWith f (x ∷ xs) ys = map (f x) ys ++ cartesianProductWith f xs ys

cartesianProduct : List A → List B → List (A × B)
cartesianProduct = cartesianProductWith _,_

------------------------------------------------------------------------
-- Aligning and zipping

alignWith : (These A B → C) → List A → List B → List C
alignWith f []       bs       = map (f ∘′ that) bs
alignWith f as       []       = map (f ∘′ this) as
alignWith f (a ∷ as) (b ∷ bs) = f (these a b) ∷ alignWith f as bs

zipWith : (A → B → C) → List A → List B → List C
zipWith f (x ∷ xs) (y ∷ ys) = f x y ∷ zipWith f xs ys
zipWith f _        _        = []

unalignWith : (A → These B C) → List A → List B × List C
unalignWith f []       = [] , []
unalignWith f (a ∷ as) with f a
... | this b    = Product.map₁ (b ∷_) (unalignWith f as)
... | that c    = Product.map₂ (c ∷_) (unalignWith f as)
... | these b c = Product.map (b ∷_) (c ∷_) (unalignWith f as)

unzipWith : (A → B × C) → List A → List B × List C
unzipWith f []         = [] , []
unzipWith f (xy ∷ xys) = Product.zip _∷_ _∷_ (f xy) (unzipWith f xys)

partitionSumsWith : (A → B ⊎ C) → List A → List B × List C
partitionSumsWith f = unalignWith (These.fromSum ∘′ f)

