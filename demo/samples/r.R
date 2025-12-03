#' Detect the presence/absence of a match
#'
#' `str_detect()` returns a logical vector with `TRUE` for each element of
#' `string` that matches `pattern` and `FALSE` otherwise. It's equivalent to
#' `grepl(pattern, string)`.
str_detect <- function(string, pattern, negate = FALSE) {
  check_lengths(string, pattern)
  check_bool(negate)

  out <- switch(
    type(pattern),
    empty = no_empty(),
    bound = no_boundary(),
    fixed = stri_detect_fixed(
      string,
      pattern,
      negate = negate,
      opts_fixed = opts(pattern)
    ),
    coll = stri_detect_coll(
      string,
      pattern,
      negate = negate,
      opts_collator = opts(pattern)
    ),
    regex = stri_detect_regex(
      string,
      pattern,
      negate = negate,
      opts_regex = opts(pattern)
    )
  )

  preserve_names_if_possible(string, pattern, out)
}

str_starts <- function(string, pattern, negate = FALSE) {
  check_lengths(string, pattern)
  check_bool(negate)

  out <- switch(
    type(pattern),
    empty = no_empty(),
    bound = no_boundary(),
    fixed = stri_startswith_fixed(
      string,
      pattern,
      negate = negate,
      opts_fixed = opts(pattern)
    ),
    coll = stri_startswith_coll(
      string,
      pattern,
      negate = negate,
      opts_collator = opts(pattern)
    ),
    regex = {
      pattern2 <- paste0("^(", pattern, ")")
      stri_detect_regex(
        string,
        pattern2,
        negate = negate,
        opts_regex = opts(pattern)
      )
    }
  )
  preserve_names_if_possible(string, pattern, out)
}

str_ends <- function(string, pattern, negate = FALSE) {
  check_lengths(string, pattern)
  check_bool(negate)

  out <- switch(
    type(pattern),
    empty = no_empty(),
    bound = no_boundary(),
    fixed = stri_endswith_fixed(
      string,
      pattern,
      negate = negate,
      opts_fixed = opts(pattern)
    ),
    coll = stri_endswith_coll(
      string,
      pattern,
      negate = negate,
      opts_collator = opts(pattern)
    ),
    regex = {
      pattern2 <- paste0("(", pattern, ")$")
      stri_detect_regex(
        string,
        pattern2,
        negate = negate,
        opts_regex = opts(pattern)
      )
    }
  )
  preserve_names_if_possible(string, pattern, out)
}

str_like <- function(string, pattern, ignore_case = deprecated()) {
  check_lengths(string, pattern)
  check_character(pattern)
  if (inherits(pattern, "stringr_pattern")) {
    cli::cli_abort(
      "{.arg pattern} must be a plain string, not a stringr modifier."
    )
  }
  if (lifecycle::is_present(ignore_case)) {
    lifecycle::deprecate_warn(
      when = "1.6.0",
      what = "str_like(ignore_case)",
      details = c(
        "`str_like()` is always case sensitive.",
        "Use `str_ilike()` for case insensitive string matching."
      )
    )
    check_bool(ignore_case)
    if (ignore_case) {
      return(str_ilike(string, pattern))
    }
  }

  pattern <- regex(like_to_regex(pattern), ignore_case = FALSE)
  out <- stri_detect_regex(string, pattern, opts_regex = opts(pattern))
  preserve_names_if_possible(string, pattern, out)
}

str_ilike <- function(string, pattern) {
  check_lengths(string, pattern)
  check_character(pattern)
  if (inherits(pattern, "stringr_pattern")) {
    cli::cli_abort(tr_(
      "{.arg pattern} must be a plain string, not a stringr modifier."
    ))
  }

  pattern <- regex(like_to_regex(pattern), ignore_case = TRUE)
  out <- stri_detect_regex(string, pattern, opts_regex = opts(pattern))
  preserve_names_if_possible(string, pattern, out)
}

like_to_regex <- function(pattern) {
  converted <- stri_replace_all_regex(
    pattern,
    "(?<!\\\\|\\[)%(?!\\])",
    "\\.\\*"
  )
  converted <- stri_replace_all_regex(converted, "(?<!\\\\|\\[)_(?!\\])", "\\.")
  paste0("^", converted, "$")
}
