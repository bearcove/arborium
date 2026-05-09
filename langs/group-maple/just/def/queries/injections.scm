; Specify nested languages that live within a `justfile`

; FIXME: these are not compatible with helix due to precedence SKIP-NVIM SKIP-NVIM-OLD

; ================ Always applicable ================

((comment) @injection.content
  (#set! injection.language "comment"))

; Highlight the RHS of `=~` as regex
((regex_literal
  (_) @injection.content)
  (#set! injection.language "regex"))

; ================ Global defaults ================

; Default everything to be bash
(recipe_body
  !shebang
  (#set! injection.language "bash")
  (#set! injection.include-children)) @injection.content

(external_command
  (command_body) @injection.content
  (#set! injection.language "bash"))

; ================ Global language specified ================
; Global language is set with something like one of the following:
;
;    set shell := ["bash", "-c", ...]
;    set shell := ["pwsh.exe"]
;
; We can extract the first item of the array, but we can't extract the language
; name from the string with something like regex. So instead we special case
; two things: powershell, which is likely to come with a `.exe` attachment that
; we need to strip, and everything else which hopefully has no extension. We
; separate this with a `#match?`.
;
; Unfortunately, there also isn't a way to allow arbitrary nesting or
; alternatively set "global" capture variables. So we can set this for item-
; level external commands, but not for e.g. external commands within an
; expression without getting _really_ annoying. Should at least look fine since
; they default to bash. Limitations...
; See https://github.com/tree-sitter/tree-sitter/issues/880 for more on that.

(source_file
  (setting "shell" ":=" "[" (string) @_langstr
    (#match? @_langstr ".*(powershell|pwsh|cmd).*")
    (#set! injection.language "powershell"))
  [
    (recipe
      (recipe_body
        !shebang
        (#set! injection.include-children)) @injection.content)

    (assignment
      (expression
        (value
          (external_command
            (command_body) @injection.content))))
  ])

(source_file
  (setting "shell" ":=" "[" (string) @injection.language
    (#not-match? @injection.language ".*(powershell|pwsh|cmd).*"))
  [
    (recipe
      (recipe_body
        !shebang
        (#set! injection.include-children)) @injection.content)

    (assignment
      (expression
        (value
          (external_command
            (command_body) @injection.content))))
  ])

; ================ Recipe language specified ================                           ; SKIP-HELIX
                                                                                        ; SKIP-HELIX
; Set highlighting for recipes that specify a language, using the exact name by default ; SKIP-HELIX
(recipe_body ;                                                                          ; SKIP-HELIX
  (shebang ;                                                                            ; SKIP-HELIX
    (language) @injection.language)                                                     ; SKIP-HELIX
  (#not-any-of? @injection.language "python3" "nodejs" "node" "uv")                     ; SKIP-HELIX
  (#set! injection.include-children)) @injection.content                                ; SKIP-HELIX
                                                                                        ; SKIP-HELIX
; Transform some known executables                                                      ; SKIP-HELIX
                                                                                        ; SKIP-HELIX
; python3/uv -> python                                                                  ; SKIP-HELIX
(recipe_body                                                                            ; SKIP-HELIX
  (shebang                                                                              ; SKIP-HELIX
    (language) @_lang)                                                                  ; SKIP-HELIX
  (#any-of? @_lang "python3" "uv")                                                      ; SKIP-HELIX
  (#set! injection.language "python")                                                   ; SKIP-HELIX
  (#set! injection.include-children)) @injection.content                                ; SKIP-HELIX
                                                                                        ; SKIP-HELIX
; node/nodejs -> javascript                                                             ; SKIP-HELIX
(recipe_body                                                                            ; SKIP-HELIX
  (shebang                                                                              ; SKIP-HELIX
    (language) @_lang)                                                                  ; SKIP-HELIX
  (#any-of? @_lang "node" "nodejs")                                                     ; SKIP-HELIX
  (#set! injection.language "javascript")                                               ; SKIP-HELIX
  (#set! injection.include-children)) @injection.content                                ; SKIP-HELIX

; ================ Recipe language specified - Helix only ================              ; SKIP-NVIM SKIP-NVIM-OLD SKIP-LAPCE SKIP-ZED
                                                                                        ; SKIP-NVIM SKIP-NVIM-OLD SKIP-LAPCE SKIP-ZED
; Set highlighting for recipes that specify a language using builtin shebang matching   ; SKIP-NVIM SKIP-NVIM-OLD SKIP-LAPCE SKIP-ZED
(recipe_body                                                                            ; SKIP-NVIM SKIP-NVIM-OLD SKIP-LAPCE SKIP-ZED
  (shebang) @injection.shebang                                                          ; SKIP-NVIM SKIP-NVIM-OLD SKIP-LAPCE SKIP-ZED
  (#set! injection.include-children)) @injection.content                                ; SKIP-NVIM SKIP-NVIM-OLD SKIP-LAPCE SKIP-ZED
