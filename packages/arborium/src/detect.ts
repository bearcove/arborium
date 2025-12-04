/**
 * Simple language detection heuristics.
 * Not meant to be comprehensive - just catches common cases.
 */

/** Shebang patterns */
const SHEBANG_PATTERNS: Array<[RegExp, string]> = [
  [/^#!.*\bpython[23]?\b/, 'python'],
  [/^#!.*\bnode\b/, 'javascript'],
  [/^#!.*\bdeno\b/, 'typescript'],
  [/^#!.*\bbun\b/, 'typescript'],
  [/^#!.*\bruby\b/, 'ruby'],
  [/^#!.*\bperl\b/, 'perl'],
  [/^#!.*\bphp\b/, 'php'],
  [/^#!.*\bbash\b/, 'bash'],
  [/^#!.*\bzsh\b/, 'zsh'],
  [/^#!.*\bsh\b/, 'bash'],
  [/^#!.*\blua\b/, 'lua'],
  [/^#!.*\bawk\b/, 'awk'],
];

/** Keyword fingerprints - first few unique keywords that identify a language */
const KEYWORD_FINGERPRINTS: Array<[RegExp, string]> = [
  // Rust - distinctive keywords
  [/\b(fn|impl|trait|pub\s+fn|let\s+mut|&mut|->)\b/, 'rust'],

  // Go - distinctive keywords
  [/\b(func|package\s+\w+|import\s+\(|go\s+func|chan\s+\w+)\b/, 'go'],

  // Python - distinctive patterns
  [/\b(def\s+\w+\s*\(|import\s+\w+|from\s+\w+\s+import|class\s+\w+:)\b/, 'python'],

  // TypeScript - distinctive type annotations
  [/:\s*(string|number|boolean|void)\b|\binterface\s+\w+\s*\{/, 'typescript'],

  // JavaScript - distinctive patterns (after TS check)
  [/\b(const|let|var)\s+\w+\s*=|function\s+\w+\s*\(|=>\s*\{/, 'javascript'],

  // Ruby - distinctive keywords
  [/\b(def\s+\w+|end\b|do\s*\|.*\||puts\s+|require\s+['"])\b/, 'ruby'],

  // Java - distinctive patterns
  [/\b(public\s+class|private\s+\w+|System\.out\.println)\b/, 'java'],

  // C++ - distinctive patterns
  [/\b(#include\s*<|std::|template\s*<|nullptr|cout\s*<<)\b/, 'cpp'],

  // C - distinctive patterns (after C++ check)
  [/\b(#include\s*[<"]|printf\s*\(|int\s+main\s*\(|void\s+\w+\s*\()\b/, 'c'],

  // C# - distinctive patterns
  [/\b(namespace\s+\w+|using\s+System|public\s+static\s+void)\b/, 'c-sharp'],

  // PHP - distinctive patterns
  [/<\?php|\$\w+\s*=/, 'php'],

  // Swift - distinctive patterns
  [/\b(func\s+\w+|var\s+\w+:\s*\w+|let\s+\w+:\s*\w+|@objc)\b/, 'swift'],

  // Kotlin - distinctive patterns
  [/\b(fun\s+\w+|val\s+\w+|var\s+\w+:|data\s+class)\b/, 'kotlin'],

  // Scala - distinctive patterns
  [/\b(def\s+\w+|val\s+\w+|var\s+\w+|object\s+\w+|case\s+class)\b/, 'scala'],

  // Haskell - distinctive patterns
  [/\b(module\s+\w+|import\s+qualified|data\s+\w+\s*=|::\s*\w+\s*->)\b/, 'haskell'],

  // Elixir - distinctive patterns
  [/\b(defmodule\s+\w+|def\s+\w+|defp\s+\w+|\|>)\b/, 'elixir'],

  // Lua - distinctive patterns
  [/\b(local\s+\w+\s*=|function\s+\w+\.\w+|require\s*\()\b/, 'lua'],

  // SQL - distinctive patterns
  [/\b(SELECT\s+.*\s+FROM|INSERT\s+INTO|CREATE\s+TABLE|ALTER\s+TABLE)\b/i, 'sql'],

  // Shell/Bash - distinctive patterns
  [/\b(if\s+\[\s*|then\b|fi\b|echo\s+["']|export\s+\w+=)\b/, 'bash'],

  // YAML - distinctive patterns
  [/^\s*[\w-]+:\s*[\w\-"'[{]|^---\s*$/, 'yaml'],

  // JSON - distinctive patterns
  [/^\s*\{[\s\S]*"[\w-]+":\s*/, 'json'],

  // TOML - distinctive patterns
  [/^\s*\[[\w.-]+\]\s*$|^\s*\w+\s*=\s*["'\d\[]/, 'toml'],

  // HTML - distinctive patterns
  [/<(!DOCTYPE|html|head|body|div|span|p|a\s)/i, 'html'],

  // CSS - distinctive patterns
  [/^\s*[\w.#@][\w\s,#.:>+~-]*\{[^}]*\}|@media\s|@import\s/, 'css'],

  // Markdown - distinctive patterns
  [/^#{1,6}\s+\w|^\s*[-*+]\s+\w|^\s*\d+\.\s+\w|```\w*\n/, 'markdown'],

  // XML - distinctive patterns
  [/<\?xml|<[\w:-]+\s+xmlns/, 'xml'],

  // Dockerfile
  [/^FROM\s+\w+|^RUN\s+|^COPY\s+|^ENTRYPOINT\s+/m, 'dockerfile'],

  // Nginx config
  [/\b(server\s*\{|location\s+[\/~]|proxy_pass\s+)\b/, 'nginx'],

  // Zig
  [/\b(pub\s+fn|const\s+\w+\s*=|@import\(|comptime)\b/, 'zig'],
];

/**
 * Detect the language of a code snippet.
 * Returns null if detection fails.
 */
export function detectLanguage(source: string): string | null {
  // Check shebang first (most reliable)
  const firstLine = source.split('\n')[0];
  for (const [pattern, language] of SHEBANG_PATTERNS) {
    if (pattern.test(firstLine)) {
      return language;
    }
  }

  // Check keyword fingerprints
  for (const [pattern, language] of KEYWORD_FINGERPRINTS) {
    if (pattern.test(source)) {
      return language;
    }
  }

  return null;
}

/**
 * Extract language from class name (e.g., "language-rust" -> "rust")
 */
export function extractLanguageFromClass(className: string): string | null {
  const match = className.match(/\blanguage-(\w+)\b/);
  return match ? match[1] : null;
}

/**
 * Normalize language identifier (handle aliases)
 */
export function normalizeLanguage(lang: string): string {
  const aliases: Record<string, string> = {
    js: 'javascript',
    ts: 'typescript',
    py: 'python',
    rb: 'ruby',
    rs: 'rust',
    sh: 'bash',
    shell: 'bash',
    yml: 'yaml',
    cs: 'c-sharp',
    csharp: 'c-sharp',
    'c++': 'cpp',
    'c#': 'c-sharp',
    'f#': 'fsharp',
    dockerfile: 'dockerfile',
    docker: 'dockerfile',
    makefile: 'make',
    plaintext: 'text',
    plain: 'text',
    txt: 'text',
  };

  const lower = lang.toLowerCase();
  return aliases[lower] || lower;
}
