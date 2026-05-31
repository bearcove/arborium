; Cirq tree-sitter locals
; ============================================================
; Captures structural scoping for params, lets, ports, and instances.

; ── Scopes ───────────────────────────────────────────────────

(circuit_decl) @local.scope
(module_decl) @local.scope
(analysis_decl) @local.scope

; ── Definitions ──────────────────────────────────────────────

; Parameters define names in their enclosing scope
(param_decl
  name: (identifier) @local.definition)

; Let bindings define names in their enclosing scope
(let_decl
  name: (identifier) @local.definition)

; Port declarations define names in their module scope
(port_decl
  name: (identifier) @local.definition)

; Global net declarations define names
(global_decl
  name: (identifier) @local.definition)

; Element instances define named components
(element_inst
  name: (identifier) @local.definition)

; Module instances define named components
(module_inst
  name: (identifier) @local.definition)

; Model declarations define named models
(model_decl
  name: (identifier) @local.definition)

; ── References ───────────────────────────────────────────────

; Identifiers in expression positions are references
(identifier) @local.reference
