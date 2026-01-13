;;; simple.el --- basic editing commands for Emacs  -*- lexical-binding: t -*-

;; Copyright (C) 1985-1987, 1993-2025 Free Software Foundation, Inc.

;; Maintainer: emacs-devel@gnu.org
;; Keywords: internal
;; Package: emacs

;; This file is part of GNU Emacs.

;; GNU Emacs is free software: you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.

;; GNU Emacs is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU General Public License for more details.

;; You should have received a copy of the GNU General Public License
;; along with GNU Emacs.  If not, see <https://www.gnu.org/licenses/>.

;;; Commentary:

;; A grab-bag of basic Emacs commands not specifically related to some
;; major mode or to file-handling.

;;; Code:

(eval-when-compile (require 'cl-lib))

(declare-function widget-apply "wid-edit" (widget property &rest args))
(declare-function widget-convert "wid-edit" (type &rest args))

;;; From compile.el
(defvar compilation-current-error)
(defvar compilation-context-lines)

(make-obsolete-variable 'idle-update-delay 'which-func-update-delay "30.1")
(defcustom idle-update-delay 0.5
  "Idle time delay before updating various things on the screen.
Various Emacs features that update auxiliary information when point moves
wait this many seconds after Emacs becomes idle before doing an update."
  :type 'number
  :group 'display
  :version "22.1")

(defvar amalgamating-undo-limit 20
  "The maximum number of changes to possibly amalgamate when undoing changes.
The `undo' command will normally consider \"similar\" changes
(like inserting characters) to be part of the same change.  This
is called \"amalgamating\" the changes.  This variable says what
the maximum number of changes considered is when amalgamating.  A
value of 1 means that nothing is amalgamated.")

(defgroup killing nil
  "Killing and yanking commands."
  :group 'editing)

(defgroup paren-matching nil
  "Highlight (un)matching of parens and expressions."
  :group 'matching)

(defvar-local escaped-string-quote "\\"
  "String to insert before a string quote character in a string to escape it.
This is typically a backslash (in most languages):

  \\='foo\\\\='bar\\='
  \"foo\\\"bar\"

But in SQL, for instance, it's \"\\='\":

  \\='foo\\='\\='bar\\='

This can also be a function, which is called with the string
terminator as the argument, and should return a string to be
used as the escape.

This variable is used by the `yank-in-context' command.")


;;; next-error support framework

(defgroup next-error nil
  "`next-error' support framework."
  :group 'compilation
  :version "22.1")

(defface next-error
  '((t (:inherit region)))
  "Face used to highlight next error locus."
  :group 'next-error
  :version "22.1")

(defcustom next-error-highlight 0.5
  "Highlighting of locations in the selected buffer.
If a number, highlight the locus in `next-error' face for the given time
in seconds, or until the next command is executed.
If t, highlight the locus until the next command is executed, or until
