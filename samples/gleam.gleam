import gleam/dict.{type Dict}
import gleam/int
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/result

/// A custom type representing the outcome of a fold operation
pub type ContinueOrStop(a) {
  Continue(a)
  Stop(a)
}

/// Counts the number of elements in a list
pub fn length(of list: List(a)) -> Int {
  do_length(list, 0)
}

fn do_length(list: List(a), acc: Int) -> Int {
  case list {
    [] -> acc
    [_, ..rest] -> do_length(rest, acc + 1)
  }
}

/// Returns the first element of a list, if any
pub fn first(list: List(a)) -> Result(a, Nil) {
  case list {
    [] -> Error(Nil)
    [x, ..] -> Ok(x)
  }
}

/// Filters a list, keeping only elements that satisfy the predicate
pub fn filter(list: List(a), keeping predicate: fn(a) -> Bool) -> List(a) {
  do_filter(list, predicate, [])
  |> reverse
}

fn do_filter(
  list: List(a),
  predicate: fn(a) -> Bool,
  acc: List(a),
) -> List(a) {
  case list {
    [] -> acc
    [x, ..rest] -> {
      let acc = case predicate(x) {
        True -> [x, ..acc]
        False -> acc
      }
      do_filter(rest, predicate, acc)
    }
  }
}

/// Reverses a list
pub fn reverse(list: List(a)) -> List(a) {
  do_reverse(list, [])
}

fn do_reverse(list: List(a), acc: List(a)) -> List(a) {
  case list {
    [] -> acc
    [x, ..rest] -> do_reverse(rest, [x, ..acc])
  }
}
