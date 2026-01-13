################
# Index macros #
################
F! ← ^0 5
G! ← F!^0
⍤⤙≍ ¯5 G!¯
Four! ← ∩∩^0
⍤⤙≍ [4 6 8 10] [Four!(×2) 2 3 4 5]
Dyad‼ ← ^0^1
⍤⤙≍ 12 Dyad‼(×2|+1) 5
F‼‼ ← ^1^0^2^2^3^0
⍤⤙≍ [1 0 2 2 3 0] F‼‼(⊂0|⊂1|⊂2|⊂3) []

F! ← (
  ^0
  ^0
)
⍤⤙≍ 6 F!+ 1 2 3
F! ← [
  ^0
  ^0
]
⍤⤙≍ [6] F!+ 1 2 3
F! ← ⨬(
  ^0
| ^0
)
⍤⤙≍ ¯5 F!¯ 0 5

F! ← -^0 i
⍤⤙≍ i F!0

# Scoping shenanigans
┌─╴M
  X  ← "Inner"
  A! ← ^0
  B! ← (^0)
└─╴
X ← "Outer"
⍤⤙≍ "Outer" M~A!X
⍤⤙≍ "Outer" M~B!X

┌─╴Mod
  X     ← "Inner"
  Mac₁! ← ^0
  Mac₂! ← Mac₁!^0
└─╴
X ← "Outer"
⍤⤙≍ "Outer" Mod~Mac₁!X
⍤⤙≍ "Outer" Mod~Mac₂!X

┌─╴Module
  Func ← (5)
└─╴
Macro! ← Module!^0
⍤⤙≍ 5 Macro!Func

┌─╴Mod
  X ← "Stuff"
└─╴
Mac! ← ^0
⍤⤙≍ "Stuff" Mac!Mod!X

┌─╴M
  X  ← 5
  F! ← +X ^0
└─╴
X ← 3
⍤⤙≍ 8 M~F!X

┌─╴Mod
  Func ↚ +1
  Mac! ← ^0 Func
└─╴
⍤⤙≍ 14 Mod~Mac!×₂ 6

┌─╴M
  Ex ~ "example"
  G‼ ← ×^1 Ex~Increment ^0
  G! ← G‼^0 2
