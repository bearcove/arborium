; This query specifies how to auto-indent logical blocks.
;
; Better documentation with diagrams is in https://docs.helix-editor.com/guides/indent.html SKIP-NVIM SKIP-NVIM-NEXT

[
  (recipe)
  (string)
  (external_command)
] @indent @extend

(comment) @indent.auto                                          ; SKIP-HELIX SKIP-ZED SKIP-LAPCE
