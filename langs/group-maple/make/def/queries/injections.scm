((shell_text) @injection.content
  (#set! injection.language "bash"))

((shell_command) @injection.content
  (#set! injection.language "bash"))
