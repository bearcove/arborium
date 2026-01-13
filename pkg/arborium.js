class k extends Error {
  constructor(e) {
    super(e), this.name = "WasiError";
  }
}
class m {
  write(e) {
    return BigInt(0);
  }
  blockingWriteAndFlush(e) {
  }
  blockingFlush() {
  }
  checkWrite() {
    return BigInt(1024 * 1024);
  }
  subscribe() {
  }
}
class x {
  read(e) {
    return new Uint8Array(0);
  }
  blockingRead(e) {
    return new Uint8Array(0);
  }
  subscribe() {
  }
}
function $() {
  const t = new m(), e = new m(), s = new x();
  return {
    "wasi:cli/environment@0.2.3": {
      getEnvironment() {
        return [];
      },
      getArguments() {
        return [];
      }
    },
    "wasi:cli/exit@0.2.3": {
      exit(n) {
        if (n.tag === "err")
          throw new k(`WASI exit with error: ${n.val}`);
      }
    },
    "wasi:cli/stdin@0.2.3": {
      getStdin() {
        return s;
      }
    },
    "wasi:cli/stdout@0.2.3": {
      getStdout() {
        return t;
      }
    },
    "wasi:cli/stderr@0.2.3": {
      getStderr() {
        return e;
      }
    },
    "wasi:clocks/wall-clock@0.2.3": {
      now() {
        const n = Date.now();
        return {
          seconds: BigInt(Math.floor(n / 1e3)),
          nanoseconds: n % 1e3 * 1e6
        };
      },
      resolution() {
        return { seconds: BigInt(0), nanoseconds: 1e6 };
      }
    },
    "wasi:filesystem/types@0.2.3": {
      // Stub - grammar plugins shouldn't use filesystem
      Descriptor: class {
      },
      DirectoryEntryStream: class {
      }
    },
    "wasi:filesystem/preopens@0.2.3": {
      getDirectories() {
        return [];
      }
    },
    "wasi:io/error@0.2.3": {
      Error: k
    },
    "wasi:io/streams@0.2.3": {
      InputStream: x,
      OutputStream: m
    },
    "wasi:random/random@0.2.3": {
      getRandomBytes(n) {
        const r = new Uint8Array(Number(n));
        return crypto.getRandomValues(r), r;
      },
      getRandomU64() {
        const n = new Uint8Array(8);
        return crypto.getRandomValues(n), new DataView(n.buffer).getBigUint64(0, !0);
      }
    }
  };
}
const L = {
  "arborium:grammar/types@0.1.0": {
    // Types are just interfaces, nothing to export
  }
}, T = {
  manual: !1,
  theme: "tokyo-night",
  selector: "pre code",
  cdn: "jsdelivr",
  version: "latest"
};
let o = { ...T }, g = null;
const y = /* @__PURE__ */ new Map();
let I = /* @__PURE__ */ new Set(), b = null, W = 1;
const d = /* @__PURE__ */ new Map();
function v() {
  return o.cdn === "jsdelivr" ? `https://cdn.jsdelivr.net/npm/@anthropic-ai/arborium@${o.version}` : o.cdn === "unpkg" ? `https://unpkg.com/@anthropic-ai/arborium@${o.version}` : o.cdn;
}
async function A() {
  if (b) return;
  const t = `${v()}/plugins.json`;
  b = await (await fetch(t)).json(), I = new Set(Object.keys(b));
}
async function E(t) {
  const e = y.get(t);
  if (e) return e;
  await A();
  const s = b?.[t];
  if (!s) return null;
  const n = v();
  try {
    const r = `${n}/${s.js}`, a = r.substring(0, r.lastIndexOf("/")), i = await import(
      /* @vite-ignore */
      r
    ), u = async (w) => {
      const c = `${a}/${w}`, f = await (await fetch(c)).arrayBuffer();
      return WebAssembly.compile(f);
    }, S = {
      ...$(),
      ...L
    }, l = (await i.instantiate(u, S)).plugin, j = {
      languageId: t,
      injectionLanguages: l.injectionLanguages?.() ?? [],
      parse: (w) => {
        const c = l.createSession();
        try {
          l.setText(c, w);
          const p = l.parse(c);
          if (p.tag === "ok")
            return p.val;
          {
            const f = p.val;
            return console.error(`Parse error: ${f.message}`), { spans: [], injections: [] };
          }
        } finally {
          l.freeSession(c);
        }
      }
    };
    return y.set(t, j), j;
  } catch (r) {
    return console.error(`Failed to load grammar plugin for ${t}:`, r), null;
  }
}
function R() {
  window.arboriumHost = {
    /** Check if a language is available (sync) */
    isLanguageAvailable(t) {
      return I.has(t) || y.has(t);
    },
    /** Load a grammar and return a handle (async) */
    async loadGrammar(t) {
      const e = await E(t);
      if (!e)
        return 0;
      for (const [n, r] of d)
        if (r === e)
          return n;
      const s = W++;
      return d.set(s, e), s;
    },
    /** Parse text using a grammar handle (sync) */
    parse(t, e) {
      const s = d.get(t);
      if (!s)
        return { spans: [], injections: [] };
      const n = s.parse(e);
      return n.injections.length > 0 && console.log(`[arborium] Language ${s.languageId} returned ${n.injections.length} injections:`, n.injections), n;
    }
  };
}
async function U() {
  if (g) return;
  R();
  const e = `${v()}/arborium_host.js`;
  try {
    const s = await import(
      /* @vite-ignore */
      e
    );
    await s.default(), g = {
      highlight: s.highlight,
      isLanguageAvailable: s.isLanguageAvailable
    };
  } catch (s) {
    throw console.error("Failed to load arborium host:", s), s;
  }
}
async function F(t, e, s) {
  if (await U(), !g) throw new Error("Host not loaded");
  return g.highlight(t, e);
}
async function M(t, e) {
  const s = await E(t);
  return s ? {
    languageId: () => s.languageId,
    injectionLanguages: () => s.injectionLanguages,
    highlight: (n) => {
      const r = s.parse(n);
      return C(n, r.spans);
    },
    parse: (n) => s.parse(n),
    dispose: () => {
    }
  } : null;
}
function C(t, e) {
  const s = [...e].sort((a, i) => a.start - i.start);
  let n = "", r = 0;
  for (const a of s) {
    a.start > r && (n += h(t.slice(r, a.start)));
    const i = B(a.capture), u = h(t.slice(a.start, a.end));
    i ? n += `<a-${i}>${u}</a-${i}>` : n += u, r = a.end;
  }
  return r < t.length && (n += h(t.slice(r))), n;
}
function B(t) {
  return t.startsWith("keyword") || t === "include" || t === "conditional" ? "k" : t.startsWith("function") || t.startsWith("method") ? "f" : t.startsWith("string") || t === "character" ? "s" : t.startsWith("comment") ? "c" : t.startsWith("type") ? "t" : t.startsWith("variable") ? "v" : t.startsWith("number") || t === "float" ? "n" : t.startsWith("operator") ? "o" : t.startsWith("punctuation") ? "p" : null;
}
function h(t) {
  return t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function H(t) {
  return t ? { ...o, ...t } : { ...o };
}
const N = [
  [/^#!.*\bpython[23]?\b/, "python"],
  [/^#!.*\bnode\b/, "javascript"],
  [/^#!.*\bdeno\b/, "typescript"],
  [/^#!.*\bbun\b/, "typescript"],
  [/^#!.*\bruby\b/, "ruby"],
  [/^#!.*\bperl\b/, "perl"],
  [/^#!.*\bphp\b/, "php"],
  [/^#!.*\bbash\b/, "bash"],
  [/^#!.*\bzsh\b/, "zsh"],
  [/^#!.*\bsh\b/, "bash"],
  [/^#!.*\blua\b/, "lua"],
  [/^#!.*\bawk\b/, "awk"]
], O = [
  // Rust - distinctive keywords
  [/\b(fn|impl|trait|pub\s+fn|let\s+mut|&mut|->)\b/, "rust"],
  // Go - distinctive keywords
  [/\b(func|package\s+\w+|import\s+\(|go\s+func|chan\s+\w+)\b/, "go"],
  // Python - distinctive patterns
  [/\b(def\s+\w+\s*\(|import\s+\w+|from\s+\w+\s+import|class\s+\w+:)\b/, "python"],
  // TypeScript - distinctive type annotations
  [/:\s*(string|number|boolean|void)\b|\binterface\s+\w+\s*\{/, "typescript"],
  // JavaScript - distinctive patterns (after TS check)
  [/\b(const|let|var)\s+\w+\s*=|function\s+\w+\s*\(|=>\s*\{/, "javascript"],
  // Ruby - distinctive keywords
  [/\b(def\s+\w+|end\b|do\s*\|.*\||puts\s+|require\s+['"])\b/, "ruby"],
  // Java - distinctive patterns
  [/\b(public\s+class|private\s+\w+|System\.out\.println)\b/, "java"],
  // C++ - distinctive patterns
  [/\b(#include\s*<|std::|template\s*<|nullptr|cout\s*<<)\b/, "cpp"],
  // C - distinctive patterns (after C++ check)
  [/\b(#include\s*[<"]|printf\s*\(|int\s+main\s*\(|void\s+\w+\s*\()\b/, "c"],
  // C# - distinctive patterns
  [/\b(namespace\s+\w+|using\s+System|public\s+static\s+void)\b/, "c-sharp"],
  // PHP - distinctive patterns
  [/<\?php|\$\w+\s*=/, "php"],
  // Swift - distinctive patterns
  [/\b(func\s+\w+|var\s+\w+:\s*\w+|let\s+\w+:\s*\w+|@objc)\b/, "swift"],
  // Kotlin - distinctive patterns
  [/\b(fun\s+\w+|val\s+\w+|var\s+\w+:|data\s+class)\b/, "kotlin"],
  // Scala - distinctive patterns
  [/\b(def\s+\w+|val\s+\w+|var\s+\w+|object\s+\w+|case\s+class)\b/, "scala"],
  // Haskell - distinctive patterns
  [/\b(module\s+\w+|import\s+qualified|data\s+\w+\s*=|::\s*\w+\s*->)\b/, "haskell"],
  // Elixir - distinctive patterns
  [/\b(defmodule\s+\w+|def\s+\w+|defp\s+\w+|\|>)\b/, "elixir"],
  // Lua - distinctive patterns
  [/\b(local\s+\w+\s*=|function\s+\w+\.\w+|require\s*\()\b/, "lua"],
  // SQL - distinctive patterns
  [/\b(SELECT\s+.*\s+FROM|INSERT\s+INTO|CREATE\s+TABLE|ALTER\s+TABLE)\b/i, "sql"],
  // Shell/Bash - distinctive patterns
  [/\b(if\s+\[\s*|then\b|fi\b|echo\s+["']|export\s+\w+=)\b/, "bash"],
  // YAML - distinctive patterns
  [/^\s*[\w-]+:\s*[\w\-"'[{]|^---\s*$/, "yaml"],
  // JSON - distinctive patterns
  [/^\s*\{[\s\S]*"[\w-]+":\s*/, "json"],
  // TOML - distinctive patterns
  [/^\s*\[[\w.-]+\]\s*$|^\s*\w+\s*=\s*["'\d\[]/, "toml"],
  // HTML - distinctive patterns
  [/<(!DOCTYPE|html|head|body|div|span|p|a\s)/i, "html"],
  // CSS - distinctive patterns
  [/^\s*[\w.#@][\w\s,#.:>+~-]*\{[^}]*\}|@media\s|@import\s/, "css"],
  // Markdown - distinctive patterns
  [/^#{1,6}\s+\w|^\s*[-*+]\s+\w|^\s*\d+\.\s+\w|```\w*\n/, "markdown"],
  // XML - distinctive patterns
  [/<\?xml|<[\w:-]+\s+xmlns/, "xml"],
  // Dockerfile
  [/^FROM\s+\w+|^RUN\s+|^COPY\s+|^ENTRYPOINT\s+/m, "dockerfile"],
  // Nginx config
  [/\b(server\s*\{|location\s+[\/~]|proxy_pass\s+)\b/, "nginx"],
  // Zig
  [/\b(pub\s+fn|const\s+\w+\s*=|@import\(|comptime)\b/, "zig"]
];
function D(t) {
  const e = t.split(`
`)[0];
  for (const [s, n] of N)
    if (s.test(e))
      return n;
  for (const [s, n] of O)
    if (s.test(t))
      return n;
  return null;
}
function G(t) {
  const e = t.match(/\blanguage-(\w+)\b/);
  return e ? e[1] : null;
}
function q(t) {
  const e = {
    js: "javascript",
    ts: "typescript",
    py: "python",
    rb: "ruby",
    rs: "rust",
    sh: "bash",
    shell: "bash",
    yml: "yaml",
    cs: "c-sharp",
    csharp: "c-sharp",
    "c++": "cpp",
    "c#": "c-sharp",
    "f#": "fsharp",
    dockerfile: "dockerfile",
    docker: "dockerfile",
    makefile: "make",
    plaintext: "text",
    plain: "text",
    txt: "text"
  }, s = t.toLowerCase();
  return e[s] || s;
}
export {
  D as detectLanguage,
  G as extractLanguageFromClass,
  H as getConfig,
  F as highlight,
  M as loadGrammar,
  q as normalizeLanguage,
  C as spansToHtml
};
//# sourceMappingURL=arborium.js.map
