/**
 * @file Batch grammar for tree-sitter
 * @author davidevofficial <davidevufficial@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-chec

module.exports = grammar({
  name: 'batch',
  rules: {
    // The top-level rule
    program: $ => repeat(choice(
      $.echooff,
      $.comment,
      $.variable_declaration,
      $.variable_reference,
      $.keyword,
      $.function_definition
    )),
    echooff: $ => seq(optional('@'),"echo off"),
    // Comments (both :: and REM)
    comment: $ => choice(
      seq(optional('@'),'::', /.*/),
      seq(optional('@'),'REM', /.*/),
      seq(optional('@'),'Rem', /.*/),
      seq(optional('@'),'rem', /.*/),
    ),
    // Variable declarations (using SET)
    variable_declaration: $ => seq(
      optional('@'),
      choice('SET', 'Set', 'set'),
      optional('/A'),
      $.identifier,
      '=',
      choice($.string, $.number, $.variable_reference) // Allow setting to a string, number, or another variable
    ),
    // Variables (anything between % symbols)
    variable_reference: $ => prec(2, seq('%', alias($.identifier, 'variable_name'), '%')),

    // Keywords (predefined list of Batch commands)
    keyword: $ => prec(1, seq(optional('@'), choice(
      "ECHO", "SET", "IF", "GOTO", "EXIT", "FOR", "REM", "PAUSE", "CLS","echo", "set", "if","goto", "exit", "for", "rem", "pause", "cls", "VER", "ASSOC", "CD", "COPY", "DEL", "DIR", "DATE", "MD", "MOVE", "PATH", "PROMPT", "RD", "REN", "START", "TIME", "TYPE", "VOL", "ATTRIB", "CHKDSK", "CHOICE", "CMD", "COMP", "CONVERT", "DRIVERQUERY", "EXPAND", "FIND", "FORMAT", "HELP", "IPCONFIG", "LABEL", "NET", "PING", "SHUTDOWN", "SORT", "SUBST", "SYSTEMINFO", "TASKKILL", "TASKLIST", "XCOPY", "TREE", "FC", "DISKPART", "TITLE", "ver", "assoc", "cd", "copy", "del", "dir", "date", "md", "move", "path", "prompt", "rd", "ren", "start", "time", "type", "vol", "attrib", "chkdsk", "choice", "cmd", "comp", "convert", "driverquery", "expand", "find", "format", "help", "ipconfig", "label", "net", "ping", "shutdown", "sort", "subst", "systeminfo", "taskkill", "tasklist", "tasklist", "xcopy", "tree", "fc", "diskpart", "title"
    ), optional(choice($.string, $.number))
    )),

    // Function definitions (labels starting with :)
    function_definition: $ => seq(
      optional('@'),
      ':',
      alias($.identifier, 'function_name')
    ),

    // Identifiers (variable names, function names)
    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // Strings (e.g., "Hello World!")
    string: $ => seq(
      '"',
      repeat(/[^"\n]/), // Match anything except a double-quote or newline
      '"'
    ),
    // Numbers (e.g., 1234)
    number: $ => /\d+/
  }
});
