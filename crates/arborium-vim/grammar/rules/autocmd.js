const { maybe_bang, keyword, commaSep1, case_insensitive } = require("./utils");

// :h events
const EVENTS = [
  "BufAdd", // after adding a buffer to the buffer list
  "BufDelete", // deleting a buffer from the buffer list
  "BufEnter", // after entering a buffer
  "BufFilePost", // after renaming a buffer
  "BufFilePre", // before renaming a buffer
  "BufHidden", // just after buffer becomes hidden
  "BufLeave", // before leaving a buffer
  "BufModifiedSet", // after the "modified" state of a buffer changes
  "BufNew", // after creating any buffer
  "BufNewFile", // when creating a buffer for a new file
  "BufReadCmd", // read buffer using command
  "BufReadPost", // after reading a buffer
  "BufReadPre", // before reading a buffer
  "BufUnload", // just before unloading a buffer
  "BufWinEnter", // after showing a buffer in a window
  "BufWinLeave", // just after buffer removed from window
  "BufWipeout", // just before really deleting a buffer
  "BufWriteCmd", // write buffer using command
  "BufWritePost", // after writing a buffer
  "BufWritePre", // before writing a buffer
  "ChanInfo", // info was received about channel
  "ChanOpen", // channel was opened
  "CmdUndefined", // command undefined
  "CmdWinEnter", // after entering the cmdline window
  "CmdWinLeave", // before leaving the cmdline window
  "CmdlineChanged", // command line was modified
  "CmdlineEnter", // after entering cmdline mode
  "CmdlineLeave", // before leaving cmdline mode
  "ColorScheme", // after loading a colorscheme
  "ColorSchemePre", // before loading a colorscheme
  "CompleteChanged", // after popup menu changed
  "CompleteDone", // after finishing insert complete
  "CompleteDonePre", // idem, before clearing info
  "CursorHold", // cursor in same position for a while
  "CursorHoldI", // idem, in Insert mode
  "CursorMoved", // cursor was moved
  "CursorMovedI", // cursor was moved in Insert mode
  "DiagnosticChanged", // diagnostics in a buffer were modified
  "DiffUpdated", // diffs have been updated
  "DirChanged", // directory changed
  "DirChangedPre", // directory is going to change
  "EncodingChanged", // after changing the "encoding" option
  "ExitPre", // before exiting
  "FileAppendCmd", // append to a file using command
  "FileAppendPost", // after appending to a file
  "FileAppendPre", // before appending to a file
  "FileChangedRO", // before first change to read-only file
  "FileChangedShell", // after shell command that changed file
  "FileChangedShellPost", // after (not) reloading changed file
  "FileReadCmd", // read from a file using command
  "FileReadPost", // after reading a file
  "FileReadPre", // before reading a file
  "FileType", // new file type detected (user defined)
  "FileWriteCmd", // write to a file using command
  "FileWritePost", // after writing a file
  "FileWritePre", // before writing a file
  "FilterReadPost", // after reading from a filter
  "FilterReadPre", // before reading from a filter
  "FilterWritePost", // after writing to a filter
  "FilterWritePre", // before writing to a filter
  "FocusGained", // got the focus
  "FocusLost", // lost the focus to another app
  "FuncUndefined", // if calling a function which doesn"t exist
  "GUIEnter", // after starting the GUI
  "GUIFailed", // after starting the GUI failed
  "InsertChange", // when changing Insert/Replace mode
  "InsertCharPre", // before inserting a char
  "InsertEnter", // when entering Insert mode
  "InsertLeave", // just after leaving Insert mode
  "InsertLeavePre", // just before leaving Insert mode
  "LspAttach", // after an LSP client attaches to a buffer
  "LspDetach", // after an LSP client detaches from a buffer
  "LspTokenUpdate", // after a visible LSP token is updated
  "MenuPopup", // just before popup menu is displayed
  "ModeChanged", // after changing the mode
  "OptionSet", // after setting any option
  "QuickFixCmdPost", // after :make, :grep etc.
  "QuickFixCmdPre", // before :make, :grep etc.
  "QuitPre", // before :quit
  "RecordingEnter", // when starting to record a macro
  "RecordingLeave", // just before a macro stops recording
  "RemoteReply", // upon string reception from a remote vim
  "SearchWrapped", // after the search wrapped around
  "SessionLoadPost", // after loading a session file
  "ShellCmdPost", // after ":!cmd"
  "ShellFilterPost", // after ":1,2!cmd", ":w !cmd", ":r !cmd".
  "Signal", // after nvim process received a signal
  "SourceCmd", // sourcing a Vim script using command
  "SourcePost", // after sourcing a Vim script
  "SourcePre", // before sourcing a Vim script
  "SpellFileMissing", // spell file missing
  "StdinReadPost", // after reading from stdin
  "StdinReadPre", // before reading from stdin
  "SwapExists", // found existing swap file
  "Syntax", // syntax selected
  "TabClosed", // a tab has closed
  "TabEnter", // after entering a tab page
  "TabLeave", // before leaving a tab page
  "TabNew", // when creating a new tab
  "TabNewEntered", // after entering a new tab
  "TermChanged", // after changing "term"
  "TermClose", // after the process exits
  "TermEnter", // after entering Terminal mode
  "TermLeave", // after leaving Terminal mode
  "TermOpen", // after opening a terminal buffer
  "TermResponse", // after setting "v:termresponse"
  "TextChanged", // text was modified
  "TextChangedI", // text was modified in Insert mode(no popup)
  "TextChangedP", // text was modified in Insert mode(popup)
  "TextChangedT", // text was modified in Terminal mode
  "TextYankPost", // after a yank or delete was done (y, d, c)
  "UIEnter", // after UI attaches
  "UILeave", // after UI detaches
  "User", // user defined autocommand
  "VimEnter", // after starting Vim
  "VimLeave", // before exiting Vim
  "VimLeavePre", // before exiting Vim and writing ShaDa file
  "VimResized", // after Vim window was resized
  "VimResume", // after Nvim is resumed
  "VimSuspend", // before Nvim is suspended
  "WinClosed", // after closing a window
  "WinEnter", // after entering a window
  "WinLeave", // before leaving a window
  "WinNew", // when entering a new window
  "WinResized", // after a window was resized
  "WinScrolled", // after a window was scrolled or resized
  "BufCreate", // alias for BufAdd
  "BufRead", // alias for BufReadPost
  "BufWrite", // alias for BufWritePre
  "FileEncoding", // alias for EncodingChanged
].map(case_insensitive);

module.exports = {
  // If an event is valid, it should take precedence over identifier
  au_event: ($) => token(prec(1, choice(...EVENTS))),
  au_event_list: ($) => commaSep1($.au_event),

  _augroup_name: ($) => alias($.identifier, $.augroup_name),

  _autocmd_pattern: ($) => commaSep1(alias(/[^ \t\n,]+/, $.pattern)),

  au_once: ($) => "++once",
  au_nested: ($) => "++nested",

  _autocmd_command: ($) =>
    seq(
      $.au_event_list,
      $._autocmd_pattern,
      optional($.au_once),
      optional($.au_nested),
      field("command", $._statement)
    ),

  // :h autocmd-define
  _autocmd_define: ($) =>
    seq(keyword($, "autocmd"), optional($._augroup_name), $._autocmd_command),

  // :h autocmd-remove
  _autocmd_remove: ($) =>
    seq(
      keyword($, "autocmd"),
      $.bang,
      optional($._augroup_name),
      optional(
        choice(
          $._autocmd_command,
          seq($.au_event_list, $._autocmd_pattern),
          $._autocmd_pattern,
          $.au_event_list
        )
      )
    ),

  // :h autocmd-list
  _autocmd_list: ($) =>
    seq(
      keyword($, "autocmd"),
      optional($._augroup_name),
      optional(
        choice(
          seq($.au_event_list, $._autocmd_pattern),
          $._autocmd_pattern,
          $.au_event_list
        )
      )
    ),

  autocmd_statement: ($) =>
    choice($._autocmd_define, $._autocmd_remove, $._autocmd_list),

  augroup_statement: ($) =>
    seq(
      maybe_bang($, keyword($, "augroup")),
      alias($.identifier, $.augroup_name)
    ),
};
