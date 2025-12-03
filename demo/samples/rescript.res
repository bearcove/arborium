@new external makeUninitializedUnsafe: int => array<'a> = "Array"
@set external truncateToLengthUnsafe: (array<'a>, int) => unit = "length"
external getUnsafe: (array<'a>, int) => 'a = "%array_unsafe_get"
external setUnsafe: (array<'a>, int, 'a) => unit = "%array_unsafe_set"

@val external fromIterator: Core__Iterator.t<'a> => array<'a> = "Array.from"
@val external fromArrayLike: Js.Array2.array_like<'a> => array<'a> = "Array.from"
@val
external fromArrayLikeWithMap: (Js.Array2.array_like<'a>, 'a => 'b) => array<'b> = "Array.from"

@send external fillAll: (array<'a>, 'a) => unit = "fill"

@send external fillToEnd: (array<'a>, 'a, ~start: int) => unit = "fill"

@send external fill: (array<'a>, 'a, ~start: int, ~end: int) => unit = "fill"

let make = (~length, x) =>
  if length <= 0 {
    []
  } else {
    let arr = makeUninitializedUnsafe(length)
    arr->fillAll(x)
    arr
  }

let fromInitializer = (~length, f) =>
  if length <= 0 {
    []
  } else {
    let arr = makeUninitializedUnsafe(length)
    for i in 0 to length - 1 {
      arr->setUnsafe(i, f(i))
    }
    arr
  }

@val external isArray: 'a => bool = "Array.isArray"

@get external length: array<'a> => int = "length"

let rec equalFromIndex = (a, b, i, eq, len) =>
  if i === len {
    true
  } else if eq(a->getUnsafe(i), b->getUnsafe(i)) {
    equalFromIndex(a, b, i + 1, eq, len)
  } else {
    false
  }

let equal = (a, b, eq) => {
  let len = a->length
  if len === b->length {
    equalFromIndex(a, b, 0, eq, len)
  } else {
    false
  }
}

let rec compareFromIndex = (a, b, i, cmp, len) =>
  if i === len {
    Core__Ordering.equal
  } else {
    let c = cmp(a->getUnsafe(i), b->getUnsafe(i))
    if c == Core__Ordering.equal {
      compareFromIndex(a, b, i + 1, cmp, len)
    } else {
      c
    }
  }

let compare = (a, b, cmp) => {
  let lenA = a->length
  let lenB = b->length
  lenA < lenB
    ? Core__Ordering.less
    : lenA > lenB
    ? Core__Ordering.greater
    : compareFromIndex(a, b, 0, cmp, lenA)
}

@send external copyAllWithin: (array<'a>, ~target: int) => array<'a> = "copyWithin"

@send
external copyWithinToEnd: (array<'a>, ~target: int, ~start: int) => array<'a> = "copyWithin"

@send
external copyWithin: (array<'a>, ~target: int, ~start: int, ~end: int) => array<'a> = "copyWithin"

@send external pop: array<'a> => option<'a> = "pop"

@send external push: (array<'a>, 'a) => unit = "push"

@variadic @send external pushMany: (array<'a>, array<'a>) => unit = "push"

@send external reverse: array<'a> => unit = "reverse"
@send external toReversed: array<'a> => array<'a> = "toReversed"

@send external shift: array<'a> => option<'a> = "shift"

@variadic @send
external splice: (array<'a>, ~start: int, ~remove: int, ~insert: array<'a>) => unit = "splice"
@variadic @send
external toSpliced: (array<'a>, ~start: int, ~remove: int, ~insert: array<'a>) => array<'a> =
  "toSpliced"

@send external with: (array<'a>, int, 'a) => array<'a> = "with"

@send external unshift: (array<'a>, 'a) => unit = "unshift"

@variadic @send external unshiftMany: (array<'a>, array<'a>) => unit = "unshift"

@send external concat: (array<'a>, array<'a>) => array<'a> = "concat"
@variadic @send external concatMany: (array<'a>, array<array<'a>>) => array<'a> = "concat"

@send external flat: array<array<'a>> => array<'a> = "flat"

@send external includes: (array<'a>, 'a) => bool = "includes"

@send external indexOf: (array<'a>, 'a) => int = "indexOf"
let indexOfOpt = (arr, item) =>
