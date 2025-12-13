// Arborium Demo - Syntax highlighting via arborium-host (wasm-bindgen)
// The Rust host handles highlighting with async grammar loading via window.arboriumHost

// Build WASI stubs for browser environment (used by grammar plugins)
function createWasiStubs() {
    class WasiInputStream {
        read(len) { return new Uint8Array(0); }
        blockingRead(len) { return new Uint8Array(0); }
    }

    class WasiOutputStream {
        write(data) { return { tag: 'ok', val: data.length }; }
        blockingFlush() {}
        blockingWriteAndFlush(data) { return { tag: 'ok', val: data.length }; }
    }

    class WasiError {}

    const stdinStream = new WasiInputStream();
    const stdoutStream = new WasiOutputStream();
    const stderrStream = new WasiOutputStream();

    return {
        'wasi:cli/environment': { getEnvironment: () => [] },
        'wasi:cli/exit': { exit: (code) => { throw new Error(`WASI exit: ${code}`); } },
        'wasi:cli/stderr': { getStderr: () => stderrStream },
        'wasi:cli/stdin': { getStdin: () => stdinStream },
        'wasi:cli/stdout': { getStdout: () => stdoutStream },
        'wasi:filesystem/preopens': { getDirectories: () => [] },
        'wasi:filesystem/types': { Descriptor: class {}, filesystemErrorCode: () => null },
        'wasi:io/error': { Error: WasiError },
        'wasi:io/streams': { InputStream: WasiInputStream, OutputStream: WasiOutputStream },
        'wasi:random/random': { getRandomBytes: (len) => new Uint8Array(Number(len)) },
    };
}

// Language metadata and manifest injected by generate-demo
const languageInfo = {
    "ada": {
        "name": "Ada",
        "tag": "code",
        "icon": "mdi:alpha-a-box-outline",
        "tier": 5,
        "description": "General-purpose systems language created for the U.S. Department of Defense; emphasizes safety and reliability.",
        "inventor": "Jean Ichbiah",
        "year": 1983,
        "url": "https://en.wikipedia.org/wiki/Ada_(programming_language)",
        "trivia": "Named after Ada Lovelace; standardized first in 1983 (Ada 83) and later revisions Ada 95, 2005, 2012, 2022."
    },
    "agda": {
        "name": "Agda",
        "tag": "code",
        "tier": 4,
        "description": "A dependently typed functional programming language and proof assistant.",
        "inventor": "Ulf Norell",
        "year": 2007,
        "url": "https://en.wikipedia.org/wiki/Agda_(programming_language)",
        "trivia": "Agda's name comes from a Swedish song about a woman named Agda who had a hen that laid eggs containing the words of the song."
    },
    "asciidoc": {
        "name": "Asciidoc",
        "tag": "markup",
        "icon": "mdi:text-box",
        "tier": 5,
        "description": "A lightweight markup language for writing technical documentation and books.",
        "inventor": "Stuart Rackham",
        "year": 2002,
        "url": "https://en.wikipedia.org/wiki/AsciiDoc",
        "trivia": "O'Reilly Media uses AsciiDoc as the source format for many of their technical books.",
        "aliases": ["adoc"]
    },
    "asm": {
        "name": "Assembly",
        "tag": "code",
        "icon": "mdi:memory",
        "tier": 3,
        "description": "Low-level programming language that maps mnemonic instructions directly to machine code; the earliest published example was Booth's 1947 ARC2 'Contracted Notation' assembler (<a href=\"https://en.wikipedia.org/wiki/Kathleen_Booth\">Kathleen Booth</a>).",
        "inventor": "Kathleen Booth",
        "year": 1947,
        "url": "https://en.wikipedia.org/wiki/Assembly_language",
        "trivia": "Booth detailed one of the first assemblers in her 1947 ARC report, and backward compatibility means the <a href=\"https://archive.org/details/Intel8086ProgrammersReferenceManual\">Intel 8086 Programmer's Reference Manual</a> is still relevant today.",
        "aliases": ["assembly"]
    },
    "awk": {
        "name": "AWK",
        "tag": "code",
        "icon": "mdi:text-box-outline",
        "tier": 4,
        "description": "Pattern-scanning and text processing language; original description in the 1978 paper 'AWK — A Pattern Scanning and Processing Language'.",
        "inventor": "Alfred Aho, Peter Weinberger, Brian Kernighan",
        "year": 1977,
        "url": "https://en.wikipedia.org/wiki/AWK",
        "trivia": "Name comes from creators' initials Aho–Weinberger–Kernighan; see the <a href=\"https://dl.acm.org/doi/10.5555/104836\">ACM paper</a> and Unix V7 manual pages for the classic implementation."
    },
    "bash": {
        "name": "Bash",
        "tag": "shell",
        "icon": "devicon-plain:bash",
        "tier": 1,
        "description": "Unix shell and command language for GNU; see the official <a href=\"https://www.gnu.org/software/bash/manual/\">Bash manual</a>.",
        "inventor": "Brian Fox",
        "year": 1989,
        "url": "https://en.wikipedia.org/wiki/Bash_(Unix_shell)",
        "trivia": "Bash means 'Bourne Again SHell', a nod to the Bourne shell; Fox began coding it in January 1988 and released the first beta in June 1989 for the GNU Project (<a href=\"https://www.gnu.org/software/bash/manual/bash.html#Bash-History\">Bash History</a>).",
        "aliases": ["sh", "shell"]
    },
    "batch": {
        "name": "Batch",
        "tag": "shell",
        "icon": "mdi:console",
        "tier": 5,
        "description": "DOS/Windows batch scripting (.bat/.cmd); commands documented in the <a href=\"https://winworldpc.com/product/ms-dos/5x\">MS-DOS 5.x manual</a> and modern <a href=\"https://learn.microsoft.com/windows-server/administration/windows-commands/windows-commands\">Windows Commands reference</a>.",
        "inventor": "Tim Paterson",
        "year": 1981,
        "url": "https://en.wikipedia.org/wiki/Batch_file",
        "trivia": "Batch files originated with 86-DOS/MS-DOS; Microsoft retained backward compatibility through cmd.exe—see the historical MS-DOS manuals for syntax.",
        "aliases": ["bat", "cmd"]
    },
    "c": {
        "name": "C",
        "tag": "code",
        "icon": "devicon-plain:c",
        "tier": 1,
        "description": "General-purpose systems language created at Bell Labs.",
        "inventor": "Dennis Ritchie",
        "year": 1972,
        "url": "https://en.wikipedia.org/wiki/C_(programming_language)",
        "trivia": "C evolved from B/BCPL during 1969-1973 alongside Unix; the name comes from its predecessor B.",
        "aliases": ["h"]
    },
    "c-sharp": {
        "name": "C#",
        "tag": "code",
        "icon": "devicon-plain:csharp",
        "tier": 1,
        "description": "Modern object-oriented language for .NET; official spec is published as <a href=\"https://www.ecma-international.org/publications-and-standards/standards/ecma-334/\">ECMA-334</a>.",
        "inventor": "Anders Hejlsberg",
        "year": 2000,
        "url": "https://en.wikipedia.org/wiki/C_Sharp_(programming_language)",
        "trivia": "Originally code-named 'Cool' (C-like Object Oriented Language) inside Microsoft before being renamed C#; see the <a href=\"https://www.ecma-international.org/publications-and-standards/standards/ecma-334/\">ECMA-334 specification</a> and early .NET announcements.",
        "aliases": ["cs", "csharp"]
    },
    "caddy": {
        "name": "Caddyfile",
        "tag": "config",
        "icon": "simple-icons:caddy",
        "tier": 5,
        "description": "Caddyfile configuration language for the Caddy web server; official docs at <a href=\"https://caddyserver.com/docs/caddyfile\">caddyserver.com</a>.",
        "inventor": "Matt Holt and Caddy contributors",
        "year": 2015,
        "url": "https://caddyserver.com/docs/",
        "trivia": "Caddy launched in 2015 with automatic HTTPS via Let's Encrypt; minimal syntax allows a single-line site definition—documented in the <a href=\"https://caddyserver.com/docs/caddyfile\">Caddyfile documentation</a>."
    },
    "capnp": {
        "name": "Cap'n Proto",
        "tag": "data",
        "icon": "mdi:message-fast-outline",
        "tier": 4,
        "description": "Cap'n Proto serialization and RPC system; design and spec at <a href=\"https://capnproto.org/\">capnproto.org</a>.",
        "inventor": "Kenton Varda",
        "year": 2013,
        "url": "https://en.wikipedia.org/wiki/Cap'n_Proto",
        "trivia": "Varda released Cap'n Proto on April 1, 2013, as a faster, zero-copy successor to Protocol Buffers; the <a href=\"https://capnproto.org/faq.html\">FAQ</a> explains the zero-copy format."
    },
    "clojure": {
        "name": "Clojure",
        "tag": "code",
        "icon": "simple-icons:clojure",
        "tier": 2,
        "description": "Functional Lisp dialect for the JVM with strong concurrency support; official site: <a href=\"https://clojure.org/\">clojure.org</a>.",
        "inventor": "Rich Hickey",
        "year": 2007,
        "url": "https://en.wikipedia.org/wiki/Clojure",
        "trivia": "Hickey designed Clojure to simplify concurrent programming; the language was announced publicly in 2007 and documented in the <a href=\"https://clojure.org/releases/downloads\">release downloads</a>.",
        "aliases": ["clj"]
    },
    "cmake": {
        "name": "CMake",
        "tag": "build",
        "icon": "devicon-plain:cmake",
        "tier": 3,
        "description": "A cross-platform build system generator for C/C++ projects.",
        "inventor": "Bill Hoffman",
        "year": 2000,
        "url": "https://en.wikipedia.org/wiki/CMake",
        "trivia": "CMake was originally developed to support the cross-platform build needs of the Insight Segmentation and Registration Toolkit (ITK)."
    },
    "commonlisp": {
        "name": "Common Lisp",
        "tag": "code",
        "icon": "mdi:lambda",
        "tier": 4,
        "description": "A multi-paradigm dialect of Lisp standardized by ANSI with powerful macro capabilities.",
        "inventor": "ANSI X3J13 committee",
        "year": 1994,
        "url": "https://en.wikipedia.org/wiki/Common_Lisp",
        "trivia": "The Common Lisp standard is over 1,000 pages long and defines nearly 1,000 symbols—making it one of the most comprehensive language specifications ever written.",
        "aliases": ["lisp", "cl"]
    },
    "cpp": {
        "name": "C++",
        "tag": "code",
        "icon": "devicon-plain:cplusplus",
        "tier": 1,
        "description": "General-purpose language extending C with zero-overhead abstractions; see Stroustrup's official overview at <a href=\"https://isocpp.org/about\">isocpp.org</a>.",
        "inventor": "Bjarne Stroustrup",
        "year": 1985,
        "url": "https://en.wikipedia.org/wiki/C%2B%2B",
        "trivia": "Began as 'C with Classes' in 1979 and was renamed C++ (using the C increment operator) in 1983; Stroustrup recounts the history in <a href=\"https://www.stroustrup.com/dne.html\">The Design and Evolution of C++</a>.",
        "aliases": ["c++", "cxx", "hpp"]
    },
    "css": {
        "name": "CSS",
        "tag": "markup",
        "icon": "devicon-plain:css3",
        "tier": 2,
        "description": "Style sheet language for web documents; the living standard is maintained by the <a href=\"https://www.w3.org/TR/CSS/\">W3C</a>.",
        "inventor": "Håkon Wium Lie",
        "year": 1996,
        "url": "https://en.wikipedia.org/wiki/CSS",
        "trivia": "Wium Lie proposed CSS on 10 Oct 1994 while at CERN and it became a W3C Recommendation in 1996; see his <a href=\"https://www.w3.org/People/howcome/p/Spec/\">original proposal</a> and the <a href=\"https://www.w3.org/TR/REC-CSS1-961217\">CSS1 specification</a>."
    },
    "d": {
        "name": "D",
        "tag": "code",
        "icon": "mdi:alpha-d-box-outline",
        "tier": 4,
        "description": "A systems programming language with C-like syntax, designed as a more modern alternative to C++.",
        "inventor": "Walter Bright",
        "year": 2001,
        "url": "https://en.wikipedia.org/wiki/D_(programming_language)",
        "trivia": "Walter Bright wrote D's first compiler entirely by himself—he previously created the first native-code C++ compiler for DOS.",
        "aliases": ["dlang"]
    },
    "dart": {
        "name": "Dart",
        "tag": "code",
        "icon": "devicon-plain:dart",
        "tier": 2,
        "description": "Object-oriented language optimized for UI development; official site: <a href=\"https://dart.dev/\">dart.dev</a>.",
        "inventor": "Lars Bak and Kasper Lund",
        "year": 2011,
        "url": "https://en.wikipedia.org/wiki/Dart_(programming_language)",
        "trivia": "Unveiled at GOTO Aarhus 2011 by Bak and Lund; the <a href=\"https://dart.dev/guides/language/language-tour\">language tour</a> documents its core goals, and Dart became the language behind Flutter when Google released Flutter in May 2017."
    },
    "devicetree": {
        "name": "Device Tree",
        "tag": "config",
        "icon": "mdi:file-tree",
        "tier": 5,
        "description": "Data structure for describing hardware; documented in the <a href=\"https://www.devicetree.org/specifications/\">Device Tree specifications</a>.",
        "inventor": "Sun Microsystems (Open Firmware)",
        "year": 1988,
        "url": "https://en.wikipedia.org/wiki/Devicetree",
        "trivia": "Originated with Open Firmware (IEEE 1275-1994); the Flattened Device Tree format for Linux was introduced in 2005—see the <a href=\"https://www.devicetree.org/specifications/\">specifications</a>."
    },
    "diff": {
        "name": "Diff",
        "tag": "data",
        "icon": "mdi:file-compare",
        "tier": 3,
        "description": "Unified diff format for representing file differences; GNU diffutils manual documents the format (<a href=\"https://www.gnu.org/software/diffutils/manual/\">diffutils manual</a>).",
        "inventor": "Wayne Davison",
        "year": 1990,
        "url": "https://en.wikipedia.org/wiki/Diff",
        "trivia": "Wayne Davison proposed the unified format in Aug 1990 on comp.sources.misc; GNU diff added support in version 1.15 (Jan 1991) per the <a href=\"https://git.savannah.gnu.org/gitweb/?p=diffutils.git;a=blob;f=NEWS;hb=refs/tags/v1.15\">NEWS file</a>.",
        "aliases": ["patch"]
    },
    "dockerfile": {
        "name": "Dockerfile",
        "tag": "config",
        "icon": "devicon-plain:docker",
        "tier": 3,
        "description": "Declarative format for Docker image builds; official reference: <a href=\"https://docs.docker.com/engine/reference/builder/\">Dockerfile reference</a>.",
        "inventor": "Solomon Hykes and Docker, Inc.",
        "year": 2013,
        "url": "https://docs.docker.com/engine/reference/builder/",
        "trivia": "Introduced with Docker's first public release in 2013; the <a href=\"https://docs.docker.com/engine/reference/builder/\">reference</a> documents each instruction and the layer-caching model that made Docker adoption explode.",
        "aliases": ["docker"]
    },
    "dot": {
        "name": "DOT/Graphviz",
        "tag": "data",
        "icon": "mdi:graph",
        "tier": 4,
        "description": "A graph description language for defining nodes, edges, and graph layouts in Graphviz.",
        "inventor": "Stephen North and AT&T Bell Labs",
        "year": 1991,
        "url": "https://en.wikipedia.org/wiki/DOT_(graph_description_language)",
        "trivia": "Graphviz layout algorithms were developed at AT&T Labs and have been used to visualize everything from compiler internals to social networks."
    },
    "elisp": {
        "name": "Emacs Lisp",
        "tag": "code",
        "icon": "simple-icons:gnuemacs",
        "tier": 4,
        "description": "A Lisp dialect used to extend and customize the GNU Emacs text editor.",
        "inventor": "Richard Stallman",
        "year": 1985,
        "url": "https://en.wikipedia.org/wiki/Emacs_Lisp",
        "trivia": "Emacs Lisp has been called 'the most widely used Lisp dialect' due to Emacs's popularity—millions of lines of Elisp packages exist.",
        "aliases": ["emacs-lisp", "el"]
    },
    "elixir": {
        "name": "Elixir",
        "tag": "code",
        "icon": "devicon-plain:elixir",
        "tier": 1,
        "description": "Functional, concurrent language on the BEAM VM; <a href=\"https://elixir-lang.org/\">official site</a>.",
        "inventor": "José Valim",
        "year": 2011,
        "url": "https://en.wikipedia.org/wiki/Elixir_(programming_language)",
        "trivia": "Valim announced Elixir in 2011 to blend Ruby-like ergonomics with Erlang's fault-tolerance; see the <a href=\"https://elixir-lang.org/getting-started/introduction.html\">getting started guide</a> and the original <a href=\"https://elixir-lang.org/blog/2011/01/09/introducing-elixir/\">announcement blog post</a>.",
        "aliases": ["ex", "exs"]
    },
    "elm": {
        "name": "Elm",
        "tag": "code",
        "icon": "devicon-plain:elm",
        "tier": 2,
        "description": "Purely functional language for reliable web apps; see the official <a href=\"https://guide.elm-lang.org/\">guide</a>.",
        "inventor": "Evan Czaplicki",
        "year": 2012,
        "url": "https://en.wikipedia.org/wiki/Elm_(programming_language)",
        "trivia": "Introduced by Czaplicki in 2012 with 'The Elm Architecture' that later inspired Redux; documented in the <a href=\"https://guide.elm-lang.org/architecture/\">architecture section</a> of the guide."
    },
    "erlang": {
        "name": "Erlang",
        "tag": "code",
        "icon": "devicon-plain:erlang",
        "tier": 2,
        "description": "A concurrent, functional language designed for building fault-tolerant distributed systems.",
        "inventor": "Joe Armstrong, Robert Virding, Mike Williams",
        "year": 1986,
        "url": "https://en.wikipedia.org/wiki/Erlang_(programming_language)",
        "trivia": "Erlang was developed at Ericsson for telecom switches—it achieved 'nine nines' (99.9999999%) uptime in production systems.",
        "aliases": ["erl"]
    },
    "fish": {
        "name": "Fish",
        "tag": "shell",
        "icon": "mdi:fish",
        "tier": 4,
        "description": "A user-friendly interactive shell with syntax highlighting and autosuggestions out of the box.",
        "inventor": "Axel Liljencrantz",
        "year": 2005,
        "url": "https://en.wikipedia.org/wiki/Friendly_interactive_shell",
        "trivia": "Fish intentionally breaks POSIX compatibility for a cleaner syntax—its slogan is 'Finally, a command line shell for the 90s!'"
    },
    "fsharp": {
        "name": "F#",
        "tag": "code",
        "icon": "devicon-plain:fsharp",
        "tier": 2,
        "description": "Functional-first .NET language with type inference; <a href=\"https://learn.microsoft.com/dotnet/fsharp/\">official docs</a>.",
        "inventor": "Don Syme",
        "year": 2005,
        "url": "https://en.wikipedia.org/wiki/F_Sharp_(programming_language)",
        "trivia": "Developed at Microsoft Research and first released publicly in 2005; see Syme's early <a href=\"https://learn.microsoft.com/dotnet/fsharp/language-reference/\">language reference</a> and the 2010 <a href=\"https://learn.microsoft.com/dotnet/fsharp/language-reference/language-specification\">language specification</a>.",
        "aliases": ["fs", "f#"]
    },
    "gleam": {
        "name": "Gleam",
        "tag": "code",
        "icon": "devicon-plain:gleam",
        "tier": 1,
        "description": "Statically typed functional language for the BEAM; <a href=\"https://gleam.run/\">official docs</a>.",
        "inventor": "Louis Pilfold",
        "year": 2016,
        "url": "https://en.wikipedia.org/wiki/Gleam_(programming_language)",
        "trivia": "Pilfold announced Gleam in 2016 to bring ML-style typing to Erlang; version 1.0 shipped in March 2024 per the <a href=\"https://gleam.run/news/gleam-v1.0/\">v1.0 announcement</a>."
    },
    "glsl": {
        "name": "GLSL",
        "tag": "code",
        "icon": "devicon-plain:opengl",
        "tier": 3,
        "description": "OpenGL Shading Language for programmable GPUs; defined in the <a href=\"https://registry.khronos.org/OpenGL/specs/gl/GLSLangSpec.4.60.pdf\">GLSL 4.60 specification</a>.",
        "inventor": "OpenGL ARB",
        "year": 2004,
        "url": "https://en.wikipedia.org/wiki/OpenGL_Shading_Language",
        "trivia": "Introduced with OpenGL 2.0 to replace the fixed pipeline; the Khronos/ARB spec (e.g., version 4.60) is the authoritative reference.",
        "aliases": ["vert", "frag"]
    },
    "go": {
        "name": "Go",
        "tag": "code",
        "icon": "devicon-plain:go",
        "tier": 1,
        "description": "Statically typed compiled language from Google; the canonical <a href=\"https://go.dev/ref/spec\">language specification</a>.",
        "inventor": "Robert Griesemer, Rob Pike, Ken Thompson",
        "year": 2009,
        "url": "https://en.wikipedia.org/wiki/Go_(programming_language)",
        "trivia": "Announced publicly in Nov 2009; its mascot the Gopher was created by Renée French and documented on the <a href=\"https://go.dev/blog/gopher\">Go blog</a>.",
        "aliases": ["golang"]
    },
    "graphql": {
        "name": "GraphQL",
        "tag": "query",
        "icon": "devicon-plain:graphql",
        "tier": 3,
        "description": "A query language for APIs that lets clients request exactly the data they need.",
        "inventor": "Lee Byron, Nick Schrock",
        "year": 2012,
        "url": "https://en.wikipedia.org/wiki/GraphQL",
        "trivia": "GraphQL was developed internally at Facebook starting in 2012 to power the News Feed on mobile apps before being open-sourced in 2015.",
        "aliases": ["gql"]
    },
    "haskell": {
        "name": "Haskell",
        "tag": "code",
        "icon": "devicon-plain:haskell",
        "tier": 1,
        "description": "Purely functional language with lazy evaluation and strong typing; see the <a href=\"https://www.haskell.org/onlinereport/haskell2010/\">Haskell 2010 Report</a> for the language definition.",
        "inventor": "Simon Peyton Jones, Paul Hudak, Philip Wadler, et al.",
        "year": 1990,
        "url": "https://en.wikipedia.org/wiki/Haskell",
        "trivia": "Named after logician Haskell Curry; standardized in 1990 by a committee that published successive language reports—the 2010 Report remains the latest official spec.",
        "aliases": ["hs"]
    },
    "hcl": {
        "name": "HCL",
        "tag": "config",
        "icon": "devicon-plain:terraform",
        "tier": 3,
        "description": "HashiCorp Configuration Language for IaC; <a href=\"https://developer.hashicorp.com/terraform/language/syntax/configuration\">language spec</a> lives in the Terraform docs.",
        "inventor": "HashiCorp",
        "year": 2014,
        "url": "https://github.com/hashicorp/hcl",
        "trivia": "Designed to be both human-readable and machine-friendly; Terraform accepts both JSON and native HCL, documented in the <a href=\"https://developer.hashicorp.com/terraform/language\">Terraform language guide</a>.",
        "aliases": ["terraform", "tf"]
    },
    "hlsl": {
        "name": "HLSL",
        "tag": "code",
        "icon": "mdi:cube-outline",
        "tier": 3,
        "description": "High-Level Shading Language for DirectX; <a href=\"https://learn.microsoft.com/windows/win32/direct3dhlsl/dx-graphics-hlsl-pguide\">official reference</a>.",
        "inventor": "Microsoft",
        "year": 2002,
        "url": "https://en.wikipedia.org/wiki/High-Level_Shading_Language",
        "trivia": "Introduced with DirectX 9; modern compiler `dxc` targets DXIL and SPIR-V—see <a href=\"https://github.com/microsoft/DirectXShaderCompiler\">DirectX Shader Compiler</a>."
    },
    "html": {
        "name": "HTML",
        "tag": "markup",
        "icon": "devicon-plain:html5",
        "tier": 2,
        "description": "Standard markup language for web documents; current <a href=\"https://html.spec.whatwg.org/\">living standard</a> is maintained by WHATWG.",
        "inventor": "Tim Berners-Lee",
        "year": 1993,
        "url": "https://en.wikipedia.org/wiki/HTML",
        "trivia": "Berners-Lee introduced HTML in 1991; the first public description appeared in 1993, and the modern living standard is maintained by <a href=\"https://whatwg.org/\">WHATWG</a> as the authoritative spec.",
        "aliases": ["htm"]
    },
    "idris": {
        "name": "Idris",
        "tag": "code",
        "icon": "mdi:alpha-i-box-outline",
        "tier": 4,
        "description": "Dependently typed functional language for verified software; see the <a href=\"https://www.idris-lang.org/\">official site</a>.",
        "inventor": "Edwin Brady",
        "year": 2009,
        "url": "https://en.wikipedia.org/wiki/Idris_(programming_language)",
        "trivia": "Brady introduced Idris in 2009 to bring dependent types to general-purpose programming; the <a href=\"https://www.idris-lang.org/documentation/\">documentation</a> demonstrates length-indexed vectors and other proofs in code.",
        "aliases": ["idr"]
    },
    "ini": {
        "name": "INI",
        "tag": "config",
        "icon": "mdi:cog-outline",
        "tier": 3,
        "description": "Simple key-value configuration format; Microsoft documented the `.ini` structure in early Windows SDKs (e.g., <a href=\"https://win16.org/\">Win16 documentation</a>).",
        "inventor": "Unknown (popularized by Microsoft Windows)",
        "year": 1985,
        "url": "https://en.wikipedia.org/wiki/INI_file",
        "trivia": "Windows 1.0 (1985) popularized `.ini` for application settings; the legacy WinAPI functions `GetPrivateProfileString` etc. are still documented in <a href=\"https://learn.microsoft.com/windows/win32/api/winbase/\">Windows API documentation</a>.",
        "aliases": ["conf", "cfg"]
    },
    "java": {
        "name": "Java",
        "tag": "code",
        "icon": "devicon-plain:java",
        "tier": 1,
        "description": "Object-oriented, class-based language targeting the JVM; authoritative <a href=\"https://docs.oracle.com/javase/specs/\">Java Language Specification</a>.",
        "inventor": "James Gosling",
        "year": 1995,
        "url": "https://en.wikipedia.org/wiki/Java_(programming_language)",
        "trivia": "Started as 'Oak' then renamed Java after Java coffee; the 'write once, run anywhere' mantra comes from Sun's 1995 launch. The <a href=\"https://docs.oracle.com/javase/specs/\">Java Language Specification</a> defines the language across versions."
    },
    "javascript": {
        "name": "JavaScript",
        "tag": "code",
        "icon": "devicon-plain:javascript",
        "tier": 1,
        "description": "High-level, dynamic language for the web and beyond; the current <a href=\"https://tc39.es/ecma262/\">ECMAScript specification</a>.",
        "inventor": "Brendan Eich",
        "year": 1995,
        "url": "https://en.wikipedia.org/wiki/JavaScript",
        "trivia": "Eich created JavaScript in 10 days at Netscape in May 1995; it shipped in Netscape Navigator 2.0. TC39 now standardizes it via <a href=\"https://tc39.es/ecma262/\">ECMAScript specification</a>, and the official name of the language is ECMAScript.",
        "aliases": ["js", "jsx", "mjs", "cjs"]
    },
    "jinja2": {
        "name": "Jinja2",
        "tag": "markup",
        "icon": "mdi:code-braces",
        "tier": 5,
        "description": "Python templating engine inspired by Django; <a href=\"https://jinja.palletsprojects.com/\">official docs</a>.",
        "inventor": "Armin Ronacher",
        "year": 2008,
        "url": "https://en.wikipedia.org/wiki/Jinja_(template_engine)",
        "trivia": "Named after the Jinja shrine; released as part of the Pocoo suite. See the <a href=\"https://jinja.palletsprojects.com/en/stable/#design-goals\">design goals</a> and early release history in the docs.",
        "aliases": ["jinja", "j2"]
    },
    "jq": {
        "name": "jq",
        "tag": "query",
        "icon": "mdi:code-json",
        "tier": 4,
        "description": "Command-line JSON processor; <a href=\"https://stedolan.github.io/jq/manual/\">manual</a> at the official site.",
        "inventor": "Stephen Dolan",
        "year": 2012,
        "url": "https://en.wikipedia.org/wiki/Jq_(programming_language)",
        "trivia": "jq debuted in 2012 inspired by AWK/sed but for JSON; language features and functions are specified in the <a href=\"https://stedolan.github.io/jq/manual/\">manual</a>."
    },
    "json": {
        "name": "JSON",
        "tag": "data",
        "icon": "mdi:code-json",
        "tier": 2,
        "description": "Lightweight data interchange format; see the <a href=\"https://www.ecma-international.org/publications-and-standards/standards/ecma-404/\">ECMA-404 standard</a>.",
        "inventor": "Douglas Crockford",
        "year": 2001,
        "url": "https://en.wikipedia.org/wiki/JSON",
        "trivia": "Crockford documented JSON in 2001 while at State Software; it was standardized as <a href=\"https://www.rfc-editor.org/rfc/rfc8259\">RFC 8259</a> and <a href=\"https://www.ecma-international.org/publications-and-standards/standards/ecma-404/\">ECMA-404</a>.",
        "aliases": ["jsonc"]
    },
    "julia": {
        "name": "Julia",
        "tag": "code",
        "icon": "devicon-plain:julia",
        "tier": 1,
        "description": "A high-performance language for numerical and scientific computing with Python-like syntax.",
        "inventor": "Jeff Bezanson, Stefan Karpinski, Viral B. Shah, Alan Edelman",
        "year": 2012,
        "url": "https://en.wikipedia.org/wiki/Julia_(programming_language)",
        "trivia": "Julia solves the 'two-language problem'—you can prototype in Julia and get C-like speed without rewriting in a lower-level language.",
        "aliases": ["jl"]
    },
    "kdl": {
        "name": "KDL",
        "tag": "config",
        "icon": "mdi:file-document-outline",
        "tier": 4,
        "description": "A document language with XML-like semantics and a more human-friendly syntax.",
        "inventor": "KDL community",
        "year": 2021,
        "url": "https://kdl.dev",
        "trivia": "KDL stands for 'KDL Document Language'—the recursive acronym is intentional, inspired by GNU."
    },
    "kotlin": {
        "name": "Kotlin",
        "tag": "code",
        "icon": "devicon-plain:kotlin",
        "tier": 1,
        "description": "A modern programming language that runs on the JVM, official for Android development",
        "inventor": "JetBrains",
        "year": 2011,
        "url": "https://en.wikipedia.org/wiki/Kotlin_(programming_language)",
        "trivia": "Kotlin was named after Kotlin Island in the Gulf of Finland, near St. Petersburg, Russia, as an homage similar to Java being named after an Indonesian island. Google announced Kotlin as an official Android development language in 2017, and in 2019 made it the preferred language for Android app development.",
        "aliases": ["kt", "kts"]
    },
    "lean": {
        "name": "Lean",
        "tag": "code",
        "icon": "mdi:alpha-l-box-outline",
        "tier": 4,
        "description": "Proof assistant and functional language based on dependent type theory; <a href=\"https://lean-lang.org/\">official docs</a>.",
        "inventor": "Leonardo de Moura",
        "year": 2013,
        "url": "https://en.wikipedia.org/wiki/Lean_(proof_assistant)",
        "trivia": "Used to formalize the 2023 Polynomial Freiman–Ruzsa proof by Gowers, Green, Manners, and Tao; the Lean community formalized it in mathlib. mathlib contained over 210,000 theorems as of <a href=\"https://lean-lang.org/blog/2024/05/16/mathlib-200k/\">May 2024</a>; AlphaProof (DeepMind) achieved IMO silver-equivalent performance in <a href=\"https://www.deepmind.com/blog/alphaproof-ai-for-mathematical-reasoning\">2024</a>."
    },
    "lua": {
        "name": "Lua",
        "tag": "code",
        "icon": "devicon-plain:lua",
        "tier": 1,
        "description": "Lightweight embeddable scripting language from PUC-Rio; <a href=\"https://www.lua.org/manual/5.4/\">official reference manual</a>.",
        "inventor": "Roberto Ierusalimschy, Waldemar Celes, Luiz Henrique de Figueiredo",
        "year": 1993,
        "url": "https://en.wikipedia.org/wiki/Lua_(programming_language)",
        "trivia": "Lua (meaning 'moon' in Portuguese) succeeded SOL ('sun'); see the <a href=\"https://www.lua.org/history.html\">history page</a> and the 1993 technical report introducing Lua."
    },
    "markdown": {
        "name": "Markdown",
        "tag": "markup",
        "icon": "simple-icons:markdown",
        "tier": 2,
        "description": "Lightweight markup language for plain-text formatting; original spec on <a href=\"https://daringfireball.net/projects/markdown/\">Gruber's site</a>.",
        "inventor": "John Gruber and Aaron Swartz",
        "year": 2004,
        "url": "https://en.wikipedia.org/wiki/Markdown",
        "trivia": "Gruber published Markdown 1.0 with Swartz as beta tester in 2004; the canonical description and reference implementation are on <a href=\"https://daringfireball.net/projects/markdown/\">Gruber's site</a>.",
        "aliases": ["md", "mdx"]
    },
    "matlab": {
        "name": "MATLAB",
        "tag": "code",
        "icon": "devicon-plain:matlab",
        "tier": 4,
        "description": "A numerical computing environment and programming language for matrix operations and data visualization.",
        "inventor": "Cleve Moler",
        "year": 1984,
        "url": "https://en.wikipedia.org/wiki/MATLAB",
        "trivia": "MATLAB was originally written to give students access to LINPACK and EISPACK without learning Fortran—the name means 'MATrix LABoratory.'",
        "aliases": ["m"]
    },
    "meson": {
        "name": "Meson",
        "tag": "build",
        "icon": "mdi:hammer-wrench",
        "tier": 4,
        "description": "Fast, user-friendly build system using Ninja backend; official docs at <a href=\"https://mesonbuild.com/Manual.html\">mesonbuild.com</a>.",
        "inventor": "Jussi Pakkanen",
        "year": 2013,
        "url": "https://en.wikipedia.org/wiki/Meson_(software)",
        "trivia": "Pakkanen published Meson in Feb 2013; see the <a href=\"https://nibblestew.blogspot.com/2013/02/introducing-meson-build-system.html\">announcement blog post</a> and the manual highlighting sub-1s configure times."
    },
    "nginx": {
        "name": "nginx",
        "tag": "config",
        "icon": "simple-icons:nginx",
        "tier": 5,
        "description": "Configuration language for the nginx web server; official reference at <a href=\"https://nginx.org/en/docs/dirindex.html\">nginx.org</a>.",
        "inventor": "Igor Sysoev",
        "year": 2004,
        "url": "https://en.wikipedia.org/wiki/Nginx",
        "trivia": "Sysoev began nginx in 2002; first public release in 2004. The configuration syntax is documented in the <a href=\"https://nginx.org/en/docs/\">official documentation</a>."
    },
    "ninja": {
        "name": "Ninja",
        "tag": "build",
        "icon": "mdi:ninja",
        "tier": 4,
        "description": "A small, fast build system focused on speed, designed as a backend for higher-level build systems.",
        "inventor": "Evan Martin",
        "year": 2011,
        "url": "https://ninja-build.org",
        "trivia": "Ninja was created at Google to speed up Chrome builds—it's intentionally minimal and meant to be generated by tools like CMake or Meson."
    },
    "nix": {
        "name": "Nix",
        "tag": "config",
        "icon": "devicon-plain:nixos",
        "tier": 2,
        "description": "Purely functional package manager and language for reproducible builds; see <a href=\"https://edolstra.github.io/pubs/phd-thesis.pdf\">Dolstra's PhD thesis</a>.",
        "inventor": "Eelco Dolstra",
        "year": 2003,
        "url": "https://en.wikipedia.org/wiki/Nix_(package_manager)",
        "trivia": "Nix stores packages in hash-addressed store paths (e.g., /nix/store/...), eliminating dependency hell. The design is formalized in Dolstra's 2006 thesis and underpins NixOS."
    },
    "objc": {
        "name": "Objective-C",
        "tag": "code",
        "icon": "devicon-plain:objectivec",
        "tier": 3,
        "description": "Object-oriented superset of C used for macOS/iOS; Apple's modern reference is the <a href=\"https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ProgrammingWithObjectiveC/\">Programming with Objective-C guide</a>.",
        "inventor": "Brad Cox and Tom Love",
        "year": 1984,
        "url": "https://en.wikipedia.org/wiki/Objective-C",
        "trivia": "Derived from Smalltalk messaging added to C; NeXT licensed it in 1988, leading to its adoption by Apple after 1996—documented in Apple's developer library.",
        "aliases": ["objective-c", "mm"]
    },
    "ocaml": {
        "name": "OCaml",
        "tag": "code",
        "icon": "devicon-plain:ocaml",
        "tier": 2,
        "description": "A multi-paradigm programming language with emphasis on expressiveness and safety",
        "inventor": "Xavier Leroy, Jérôme Vouillon, Damien Doligez, Didier Rémy",
        "year": 1996,
        "url": "https://en.wikipedia.org/wiki/OCaml",
        "trivia": "The 'O' in OCaml stands for 'Objective', as it was called Objective Caml before 2011. OCaml is used extensively in formal verification and financial trading systems. Jane Street, one of the largest proprietary trading firms, uses OCaml for over 30 million lines of production code.",
        "aliases": ["ml"]
    },
    "perl": {
        "name": "Perl",
        "tag": "code",
        "icon": "devicon-plain:perl",
        "tier": 2,
        "description": "General-purpose scripting language renowned for text processing; core docs at <a href=\"https://perldoc.perl.org/\">perldoc.perl.org</a>.",
        "inventor": "Larry Wall",
        "year": 1987,
        "url": "https://en.wikipedia.org/wiki/Perl",
        "trivia": "Wall released Perl 1.0 on 18 Dec 1987; the motto 'There's more than one way to do it' is from the official <a href=\"https://perldoc.perl.org/perlstyle\">perlstyle</a> and <a href=\"https://perldoc.perl.org/perlintro\">perlintro</a> documentation.",
        "aliases": ["pl", "pm"]
    },
    "php": {
        "name": "PHP",
        "tag": "code",
        "icon": "devicon-plain:php",
        "tier": 2,
        "description": "Server-side scripting language; the official manual is at <a href=\"https://www.php.net/manual/en/\">php.net</a>.",
        "inventor": "Rasmus Lerdorf",
        "year": 1995,
        "url": "https://en.wikipedia.org/wiki/PHP",
        "trivia": "Started as 'Personal Home Page Tools' in 1994/95; renamed 'PHP: Hypertext Preprocessor' in 1997. History is documented in the <a href=\"https://www.php.net/ChangeLog-8.php\">ChangeLog</a> and manual preface."
    },
    "postscript": {
        "name": "PostScript",
        "tag": "code",
        "icon": "mdi:printer",
        "tier": 5,
        "description": "Page description and programming language from Adobe; reference in the <a href=\"https://www.adobe.com/content/dam/acom/en/devnet/actionscript/articles/PLRM.pdf\">PostScript Language Reference Manual</a>.",
        "inventor": "John Warnock and Charles Geschke",
        "year": 1984,
        "url": "https://en.wikipedia.org/wiki/PostScript",
        "trivia": "Adobe released PostScript in 1984; the LaserWriter (1985) used it to ignite desktop publishing. The 'Red Book' PLRM is the definitive spec.",
        "aliases": ["ps"]
    },
    "powershell": {
        "name": "PowerShell",
        "tag": "shell",
        "icon": "devicon-plain:powershell",
        "tier": 1,
        "description": "Task automation shell and scripting language; official docs at <a href=\"https://learn.microsoft.com/powershell/\">Microsoft Learn</a>.",
        "inventor": "Jeffrey Snover",
        "year": 2006,
        "url": "https://en.wikipedia.org/wiki/PowerShell",
        "trivia": "Born from Snover's 2002 'Monad Manifesto'; first released 2006. Open-sourced and cross-platform in 2016 as PowerShell Core—see the <a href=\"https://github.com/PowerShell/PowerShell\">PowerShell repository</a>.",
        "aliases": ["ps1", "pwsh"]
    },
    "prolog": {
        "name": "Prolog",
        "tag": "code",
        "icon": "mdi:head-question",
        "tier": 4,
        "description": "Logic programming language for AI and computational linguistics; early definition in Colmerauer's 1972 Marseille reports.",
        "inventor": "Alain Colmerauer and Philippe Roussel",
        "year": 1972,
        "url": "https://en.wikipedia.org/wiki/Prolog",
        "trivia": "Prolog (PROgrammation en LOGique) emerged in 1972; used in Japan's Fifth Generation Project (1982). The ISO standard is <a href=\"https://www.iso.org/standard/21413.html\">ISO/IEC 13211-1:1995</a>.",
        "aliases": ["pl"]
    },
    "python": {
        "name": "Python",
        "tag": "code",
        "icon": "devicon-plain:python",
        "tier": 1,
        "description": "High-level general-purpose language emphasizing readability; authoritative reference is the <a href=\"https://docs.python.org/3/reference/\">Python Language Reference</a>.",
        "inventor": "Guido van Rossum",
        "year": 1991,
        "url": "https://en.wikipedia.org/wiki/Python_(programming_language)",
        "trivia": "Named after Monty Python; van Rossum released Python 0.9.0 in Feb 1991. The 'Zen of Python' is documented in `import this` and <a href=\"https://peps.python.org/pep-0020/\">PEP 20</a>.",
        "aliases": ["py", "py3", "python3"]
    },
    "query": {
        "name": "Tree-sitter Query",
        "tag": "query",
        "icon": "mdi:file-tree-outline",
        "tier": 5,
        "description": "A S-expression based pattern language for matching nodes in tree-sitter syntax trees.",
        "inventor": "Max Brunsfeld",
        "year": 2019,
        "url": "https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries",
        "trivia": "Tree-sitter queries power syntax highlighting in editors like Neovim, Helix, and Zed—this is what arborium uses under the hood.",
        "aliases": ["scm"]
    },
    "r": {
        "name": "R",
        "tag": "code",
        "icon": "devicon-plain:r",
        "tier": 1,
        "description": "Language and environment for statistical computing; official manuals at <a href=\"https://cran.r-project.org/manuals.html\">CRAN</a>.",
        "inventor": "Ross Ihaka and Robert Gentleman",
        "year": 1993,
        "url": "https://en.wikipedia.org/wiki/R_(programming_language)",
        "trivia": "Named for Ross and Robert and as a successor to S; initial public release was 1995. The <a href=\"https://cran.r-project.org/doc/manuals/r-release/R-lang.html\">R Language Definition</a> documents core semantics.",
        "aliases": ["rlang"]
    },
    "rescript": {
        "name": "ReScript",
        "tag": "code",
        "icon": "simple-icons:rescript",
        "tier": 4,
        "description": "Functional language that compiles to JavaScript, evolved from BuckleScript/Reason; docs at <a href=\"https://rescript-lang.org/docs/latest\">rescript-lang.org</a>.",
        "inventor": "Hongbo Zhang and the ReScript team",
        "year": 2020,
        "url": "https://rescript-lang.org/",
        "trivia": "Rebranded from BuckleScript/Reason in 2020 to focus on a JS-centric pipeline; see the <a href=\"https://rescript-lang.org/blog/announcement\">announcement</a> and docs for the syntax and compiler.",
        "aliases": ["res"]
    },
    "ron": {
        "name": "RON",
        "tag": "data",
        "icon": "devicon-plain:rust",
        "tier": 4,
        "description": "Rusty Object Notation, a Rust-friendly data format; reference at <a href=\"https://docs.rs/ron/latest/ron/\">docs.rs/ron</a>.",
        "inventor": "kvark and torkleyy",
        "year": 2015,
        "url": "https://github.com/ron-rs/ron",
        "trivia": "Designed for a (canceled) game project and revived by the Rust community; features trailing commas, comments, and Rust-like enums—see the <a href=\"https://docs.rs/ron\">crate docs</a>."
    },
    "ruby": {
        "name": "Ruby",
        "tag": "code",
        "icon": "devicon-plain:ruby",
        "tier": 1,
        "description": "A dynamic, object-oriented language designed for programmer happiness and productivity.",
        "inventor": "Yukihiro Matsumoto",
        "year": 1995,
        "url": "https://en.wikipedia.org/wiki/Ruby_(programming_language)",
        "trivia": "Matz (Yukihiro Matsumoto) chose the name Ruby as a playful reference to Perl—ruby is a precious stone, like pearl.",
        "aliases": ["rb"]
    },
    "rust": {
        "name": "Rust",
        "tag": "code",
        "icon": "devicon-plain:rust",
        "tier": 1,
        "description": "Systems language focused on safety and performance without GC; official docs at <a href=\"https://doc.rust-lang.org/book/\">The Rust Book</a>.",
        "inventor": "Graydon Hoare",
        "year": 2006,
        "url": "https://en.wikipedia.org/wiki/Rust_(programming_language)",
        "trivia": "Hoare began Rust as a side project at Mozilla in 2006; it became a Mozilla-backed project in 2009. The ownership/borrow checker model is documented in the <a href=\"https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html\">Understanding Ownership chapter</a>.",
        "aliases": ["rs"]
    },
    "scala": {
        "name": "Scala",
        "tag": "code",
        "icon": "devicon-plain:scala",
        "tier": 2,
        "description": "Multi-paradigm JVM language blending OO and FP; official docs at <a href=\"https://docs.scala-lang.org/\">docs.scala-lang.org</a>.",
        "inventor": "Martin Odersky",
        "year": 2004,
        "url": "https://en.wikipedia.org/wiki/Scala_(programming_language)",
        "trivia": "The name abbreviates 'scalable language'; Odersky announced Scala in 2004 after work on Java generics and the Pizza language. The <a href=\"https://docs.scala-lang.org/scala3/reference/\">Scala 3 reference</a> documents current syntax and features."
    },
    "scheme": {
        "name": "Scheme",
        "tag": "code",
        "icon": "mdi:lambda",
        "tier": 2,
        "description": "A minimalist dialect of Lisp with lexical scope, first-class procedures, and tail-call optimization",
        "inventor": "Guy L. Steele Jr., Gerald Jay Sussman",
        "year": 1975,
        "url": "https://en.wikipedia.org/wiki/Scheme_(programming_language)",
        "trivia": "Scheme was created at MIT AI Lab and influenced the design of many languages including JavaScript (which borrowed first-class functions and closures). The language was intentionally small - the original spec fit on 50 pages. Scheme's name comes from 'Schemer', after the AI language Planner, but was shortened due to filename length limits.",
        "aliases": ["scm"]
    },
    "scss": {
        "name": "SCSS",
        "tag": "markup",
        "icon": "simple-icons:sass",
        "tier": 3,
        "description": "Sass/SCSS is a CSS superset adding variables, nesting, and mixins; see the official <a href=\"https://sass-lang.com/documentation/\">documentation</a>.",
        "inventor": "Hampton Catlin, Natalie Weizenbaum, Chris Eppstein",
        "year": 2006,
        "url": "https://en.wikipedia.org/wiki/Sass_(style_sheet_language)",
        "trivia": "SCSS syntax arrived with Sass 3 (2010) to mirror CSS; history and design notes are in the <a href=\"https://sass-lang.com/blog/\">Sass blog</a> and <a href=\"https://sass-lang.com/documentation/syntax\">syntax documentation</a>.",
        "aliases": ["sass"]
    },
    "sparql": {
        "name": "SPARQL",
        "tag": "query",
        "icon": "mdi:graph-outline",
        "tier": 5,
        "description": "A query language for retrieving and manipulating data stored in RDF (Resource Description Framework) format.",
        "inventor": "W3C RDF Data Access Working Group",
        "year": 2008,
        "url": "https://en.wikipedia.org/wiki/SPARQL",
        "trivia": "SPARQL is a recursive acronym: 'SPARQL Protocol and RDF Query Language'—it powers knowledge graphs like Wikidata and DBpedia.",
        "aliases": ["rq"]
    },
    "sql": {
        "name": "SQL",
        "tag": "query",
        "icon": "mdi:database-outline",
        "tier": 2,
        "description": "Structured Query Language for relational databases; standardized by ANSI/ISO, see <a href=\"https://www.iso.org/standard/63555.html\">ISO/IEC 9075</a> (paywalled) and <a href=\"https://sqlite.org/lang.html\">SQLite documentation</a> as an open reference.",
        "inventor": "Donald D. Chamberlin and Raymond F. Boyce",
        "year": 1974,
        "url": "https://en.wikipedia.org/wiki/SQL",
        "trivia": "Initially called SEQUEL at IBM for System R; trademark issues shortened it to SQL. Early history is recounted in Chamberlin's paper 'SEQUEL: A Structured English Query Language'.",
        "aliases": ["mysql", "postgresql", "postgres", "sqlite"]
    },
    "ssh-config": {
        "name": "SSH Config",
        "tag": "config",
        "icon": "mdi:key-chain",
        "tier": 5,
        "description": "Configuration file format for OpenSSH client settings like hosts, keys, and connection options.",
        "inventor": "Tatu Ylönen",
        "year": 1995,
        "url": "https://man.openbsd.org/ssh_config",
        "trivia": "SSH was created in 1995 after a password-sniffing attack at Helsinki University of Technology prompted Tatu Ylönen to develop a secure replacement for rlogin."
    },
    "starlark": {
        "name": "Starlark",
        "tag": "build",
        "icon": "mdi:star-outline",
        "tier": 4,
        "description": "Deterministic, hermetic build language used by Bazel/Buck2; spec at <a href=\"https://bazel.build/rules/language\">bazel.build/rules/language</a>.",
        "inventor": "Laurent Le Brun and Google",
        "year": 2015,
        "url": "https://en.wikipedia.org/wiki/Starlark",
        "trivia": "Originally named Skylark (2015), renamed Starlark in 2018; deterministic semantics documented in the <a href=\"https://bazel.build/rules/language\">Bazel language guide</a>.",
        "aliases": ["bzl", "bazel"]
    },
    "svelte": {
        "name": "Svelte",
        "tag": "markup",
        "icon": "devicon-plain:svelte",
        "tier": 3,
        "description": "Compiler-based UI framework that outputs minimal JS; official tutorial at <a href=\"https://svelte.dev/docs/introduction\">svelte.dev</a>.",
        "inventor": "Rich Harris",
        "year": 2016,
        "url": "https://en.wikipedia.org/wiki/Svelte",
        "trivia": "Harris announced Svelte in 2016 to eliminate the virtual DOM; the <a href=\"https://svelte.dev/blog\">Svelte blog</a> covers milestones like SvelteKit and run-time improvements."
    },
    "swift": {
        "name": "Swift",
        "tag": "code",
        "icon": "devicon-plain:swift",
        "tier": 1,
        "description": "A powerful and intuitive programming language for Apple platforms",
        "inventor": "Chris Lattner",
        "year": 2014,
        "url": "https://en.wikipedia.org/wiki/Swift_(programming_language)",
        "trivia": "Chris Lattner started Swift as a personal project in 2010 while working at Apple, spending 18 months on it before getting approval to make it official. He also created LLVM and Clang. Swift was designed to be a safer language than Objective-C, eliminating entire categories of bugs through features like optionals and automatic memory management."
    },
    "textproto": {
        "name": "Text Proto",
        "tag": "data",
        "icon": "mdi:message-outline",
        "tier": 4,
        "description": "A human-readable text format for Protocol Buffer messages, commonly used for configuration files.",
        "inventor": "Google",
        "year": 2001,
        "url": "https://github.com/protocolbuffers/protobuf",
        "trivia": "Protocol Buffers were developed at Google in 2001 and are used for nearly all inter-machine communication at the company.",
        "aliases": ["pbtxt", "textpb"]
    },
    "thrift": {
        "name": "Thrift",
        "tag": "data",
        "icon": "mdi:swap-horizontal",
        "tier": 4,
        "description": "Interface Definition Language and RPC framework; official Apache docs at <a href=\"https://thrift.apache.org/docs\">thrift.apache.org</a>.",
        "inventor": "Facebook",
        "year": 2007,
        "url": "https://en.wikipedia.org/wiki/Apache_Thrift",
        "trivia": "Developed at Facebook starting 2006, open-sourced April 2007; entered Apache Incubator in May 2008 and became a TLP in October 2010—see the <a href=\"https://thrift.apache.org/\">Apache Thrift site</a>."
    },
    "tlaplus": {
        "name": "TLA+",
        "tag": "code",
        "icon": "mdi:math-integral",
        "tier": 4,
        "description": "Temporal Logic of Actions (TLA+) for formal specs; the canonical reference is <a href=\"https://lamport.azurewebsites.net/tla/book.html\">Lamport's book</a>.",
        "inventor": "Leslie Lamport",
        "year": 1999,
        "url": "https://en.wikipedia.org/wiki/TLA+",
        "trivia": "Lamport introduced TLA+ in the late 1990s; AWS famously uses it to find subtle bugs—see Lamport's papers and the TLA+ hyperbook.",
        "aliases": ["tla"]
    },
    "toml": {
        "name": "TOML",
        "tag": "config",
        "icon": "simple-icons:toml",
        "tier": 2,
        "description": "Tom's Obvious, Minimal Language for configuration; official spec at <a href=\"https://toml.io/en/\">toml.io</a>.",
        "inventor": "Tom Preston-Werner",
        "year": 2013,
        "url": "https://en.wikipedia.org/wiki/TOML",
        "trivia": "TOML 1.0.0 was released January 2021 after 8 years of iteration; see the spec changelog on <a href=\"https://toml.io/en/v1.0.0\">toml.io</a>."
    },
    "tsx": {
        "name": "TSX",
        "tag": "code",
        "icon": "simple-icons:react",
        "tier": 3,
        "description": "TypeScript with JSX support for React component development",
        "inventor": "Facebook (Jordan Walke for React/JSX) + Microsoft (TypeScript)",
        "year": 2013,
        "url": "https://en.wikipedia.org/wiki/TypeScript",
        "trivia": "TSX combines TypeScript's static type checking with JSX's declarative UI syntax. JSX was introduced by Facebook for React in 2013, and TypeScript added TSX support in 2015 with version 1.6. Today, TSX is the de facto standard for React development in TypeScript codebases."
    },
    "typescript": {
        "name": "TypeScript",
        "tag": "code",
        "icon": "devicon-plain:typescript",
        "tier": 1,
        "description": "Statically typed superset of JavaScript; official handbook at <a href=\"https://www.typescriptlang.org/docs/\">typescriptlang.org</a>.",
        "inventor": "Anders Hejlsberg",
        "year": 2012,
        "url": "https://en.wikipedia.org/wiki/TypeScript",
        "trivia": "Announced by Microsoft in Oct 2012 to add optional types to JS; design is documented in the TypeScript spec and Handbook.",
        "aliases": ["ts", "tsx", "mts", "cts"]
    },
    "typst": {
        "name": "Typst",
        "tag": "markup",
        "icon": "simple-icons:typst",
        "tier": 3,
        "description": "A modern markup-based typesetting system designed as a faster, friendlier alternative to LaTeX.",
        "inventor": "Laurenz Mädje and Martin Haug",
        "year": 2023,
        "url": "https://typst.app",
        "trivia": "Typst compiles documents incrementally—changes preview instantly, unlike LaTeX which must recompile the entire document.",
        "aliases": ["typ"]
    },
    "uiua": {
        "name": "Uiua",
        "tag": "code",
        "icon": "mdi:alpha-u-box-outline",
        "tier": 5,
        "description": "A stack-based array programming language with a focus on tacit (point-free) code using Unicode glyphs.",
        "inventor": "Kai Schmidt",
        "year": 2023,
        "url": "https://www.uiua.org",
        "trivia": "Uiua (pronounced 'wee-wuh') uses Unicode symbols for operations—you can type them with ASCII shortcuts that auto-convert.",
        "aliases": ["ua"]
    },
    "vb": {
        "name": "Visual Basic",
        "tag": "code",
        "icon": "devicon-plain:visualbasic",
        "tier": 5,
        "description": "Visual Basic .NET, object-oriented language for the .NET Framework; language reference at <a href=\"https://learn.microsoft.com/dotnet/visual-basic/\">learn.microsoft.com</a>.",
        "inventor": "Microsoft",
        "year": 2002,
        "url": "https://en.wikipedia.org/wiki/Visual_Basic_(.NET)",
        "trivia": "Launched with .NET Framework 1.0 in 2002 as a rewrite of VB6; the <a href=\"https://learn.microsoft.com/dotnet/visual-basic/language-reference/\">VB.NET language reference</a> documents syntax changes and OO features.",
        "aliases": ["vbnet", "visualbasic"]
    },
    "verilog": {
        "name": "Verilog",
        "tag": "code",
        "icon": "mdi:chip",
        "tier": 5,
        "description": "Hardware description language standardized as <a href=\"https://standards.ieee.org/ieee/1364/2985/\">IEEE 1364</a>.",
        "inventor": "Phil Moorby and Prabhu Goel",
        "year": 1984,
        "url": "https://en.wikipedia.org/wiki/Verilog",
        "trivia": "Created at Gateway Design Automation (later Cadence) in 1984; became an open IEEE standard in 1995 (IEEE 1364-1995).",
        "aliases": ["v", "sv", "systemverilog"]
    },
    "vhdl": {
        "name": "VHDL",
        "tag": "code",
        "icon": "mdi:chip",
        "tier": 5,
        "description": "VHSIC Hardware Description Language; standardized as <a href=\"https://standards.ieee.org/ieee/1076/7424/\">IEEE 1076</a>.",
        "inventor": "U.S. Department of Defense",
        "year": 1983,
        "url": "https://en.wikipedia.org/wiki/VHDL",
        "trivia": "Developed under the VHSIC program and standardized in 1987 as IEEE 1076; later revisions added numeric_std and child standards.",
        "aliases": ["vhd"]
    },
    "vim": {
        "name": "Vimscript",
        "tag": "config",
        "icon": "devicon-plain:vim",
        "tier": 3,
        "description": "Scripting language for the Vim editor; see the official Vim documentation at <http://vimdoc.sourceforge.net/>.",
        "inventor": "Bram Moolenaar",
        "year": 1991,
        "url": "https://en.wikipedia.org/wiki/Vim_(text_editor)",
        "trivia": "Vimscript evolved alongside Vim since 1991; Vim 7 (2006) added major scripting features. Bram maintained Vim until his passing in 2023.",
        "aliases": ["vimscript", "viml"]
    },
    "vue": {
        "name": "Vue",
        "tag": "markup",
        "icon": "devicon-plain:vuejs",
        "tier": 3,
        "description": "Progressive JavaScript framework for UIs; official docs at <a href=\"https://vuejs.org/guide/introduction.html\">vuejs.org</a>.",
        "inventor": "Evan You",
        "year": 2014,
        "url": "https://en.wikipedia.org/wiki/Vue.js",
        "trivia": "You released Vue in Feb 2014 after working on Angular at Google; <a href=\"https://github.com/vuejs/core/releases\">release history</a> and guide document its evolution from 0.10 to Vue 3/4 composition API."
    },
    "x86asm": {
        "name": "x86 Assembly",
        "tag": "code",
        "icon": "mdi:memory",
        "tier": 3,
        "description": "Assembly language family for Intel x86; reference manuals at [Intel SDM](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html).",
        "inventor": "Intel (Stephen P. Morse et al.)",
        "year": 1978,
        "url": "https://en.wikipedia.org/wiki/X86_assembly_language",
        "trivia": "Built on the 8086 ISA released June 1978; Intel's Software Developer Manuals document backward compatibility that lets 1978-era code run on modern CPUs.",
        "aliases": ["nasm", "x86"]
    },
    "xml": {
        "name": "XML",
        "tag": "markup",
        "icon": "devicon-plain:xml",
        "tier": 3,
        "description": "Extensible Markup Language defined by the W3C XML 1.0 Recommendation.",
        "inventor": "W3C XML Working Group",
        "year": 1998,
        "url": "https://en.wikipedia.org/wiki/XML",
        "trivia": "Work began in 1996; XML 1.0 became a W3C Recommendation on Feb 10, 1998, led by Bray, Paoli, and Sperberg-McQueen.",
        "aliases": ["xsl", "xslt", "svg"]
    },
    "yaml": {
        "name": "YAML",
        "tag": "config",
        "icon": "devicon-plain:yaml",
        "tier": 2,
        "description": "Human-friendly data serialization language; official 1.2 spec at <a href=\"https://yaml.org/spec/1.2.2/\">yaml.org</a>.",
        "inventor": "Clark Evans, Ingy döt Net, Oren Ben-Kiki",
        "year": 2001,
        "url": "https://en.wikipedia.org/wiki/YAML",
        "trivia": "Originally 'Yet Another Markup Language', later 'YAML Ain't Markup Language'; first proposed in 2001—see the <a href=\"https://yaml.org/spec/history/2001-01.html\">YAML history</a>.",
        "aliases": ["yml"]
    },
    "yuri": {
        "name": "Yuri",
        "tag": "code",
        "icon": "mdi:palette",
        "tier": 5,
        "description": "Yuri shader language; see repository README for current design status.",
        "inventor": "Yuri contributors",
        "year": 2024,
        "url": "https://git.gay/yuri/yuri",
        "trivia": "Yuri is an in-progress shader language and compiler; the design is not finalized and is released under Apache-2.0."
    },
    "zig": {
        "name": "Zig",
        "tag": "code",
        "icon": "simple-icons:zig",
        "description": "General-purpose systems language focused on simplicity and control; docs at <a href=\"https://ziglang.org/documentation/master/\">ziglang.org</a>.",
        "inventor": "Andrew Kelley",
        "year": 2016,
        "url": "https://en.wikipedia.org/wiki/Zig_(programming_language)",
        "trivia": "Kelley introduced Zig in 2016 as a C replacement without hidden control flow; the self-hosted compiler milestone is chronicled on the <a href=\"https://ziglang.org/news/\">Zig news</a>."
    },
    "zsh": {
        "name": "Zsh",
        "tag": "shell",
        "icon": "mdi:console",
        "tier": 3,
        "description": "Extended Unix shell with rich scripting; reference manual at <a href=\"https://zsh.sourceforge.io/Doc/\">zsh.sourceforge.io</a>.",
        "inventor": "Paul Falstad",
        "year": 1990,
        "url": "https://en.wikipedia.org/wiki/Z_shell",
        "trivia": "Created by Falstad while at Princeton; named after TA Zhong Shao. The <a href=\"https://zsh.sourceforge.io/FAQ/\">Zsh FAQ</a> documents its history and feature set."
    }
};

// Examples: maps language id to file extension (e.g. "rust" -> "rs")
// Content fetched on-demand from /samples/{id}.{ext}
const exampleExtensions = {
    "ada": "adb",
    "agda": "agda",
    "asciidoc": "adoc",
    "asm": "asm",
    "awk": "awk",
    "bash": "sh",
    "batch": "bat",
    "c": "c",
    "c-sharp": "cs",
    "caddy": "txt",
    "capnp": "capnp",
    "clojure": "clj",
    "cmake": "cmake",
    "commonlisp": "lisp",
    "cpp": "cc",
    "css": "css",
    "d": "d",
    "dart": "dart",
    "devicetree": "dts",
    "diff": "patch",
    "dockerfile": "Dockerfile",
    "dot": "gv",
    "elisp": "el",
    "elixir": "ex",
    "elm": "elm",
    "erlang": "erl",
    "fish": "fish",
    "fsharp": "fs",
    "gleam": "gleam",
    "glsl": "frag",
    "go": "go",
    "graphql": "graphql",
    "haskell": "hs",
    "hcl": "tf",
    "hlsl": "hlsl",
    "html": "html",
    "idris": "idr",
    "ini": "ini",
    "java": "java",
    "javascript": "js",
    "jinja2": "html",
    "jq": "jq",
    "json": "json",
    "julia": "jl",
    "kdl": "kdl",
    "kotlin": "kt",
    "lean": "lean",
    "lua": "lua",
    "markdown": "md",
    "matlab": "m",
    "meson": "build",
    "nginx": "conf",
    "ninja": "ninja",
    "nix": "nix",
    "objc": "mm",
    "ocaml": "ml",
    "perl": "pm",
    "php": "php",
    "postscript": "ps",
    "powershell": "ps1",
    "prolog": "pl",
    "python": "py",
    "query": "scm",
    "r": "R",
    "rescript": "res",
    "ron": "ron",
    "ruby": "rb",
    "rust": "rs",
    "scala": "scala",
    "scheme": "rkt",
    "scss": "scss",
    "sparql": "sparql",
    "sql": "sql",
    "ssh-config": "txt",
    "starlark": "bzl",
    "svelte": "svelte",
    "swift": "swift",
    "textproto": "textproto",
    "thrift": "thrift",
    "tlaplus": "tla",
    "toml": "toml",
    "tsx": "tsx",
    "typescript": "ts",
    "typst": "typ",
    "uiua": "ua",
    "vb": "vb",
    "verilog": "sv",
    "vhdl": "vhdl",
    "vim": "vim",
    "vue": "vue",
    "x86asm": "asm",
    "xml": "xml",
    "yaml": "yaml",
    "yuri": "yuri",
    "zig": "zig",
    "zsh": "zsh"
};

// Icons will be injected by generate-demo (SVG strings keyed by iconify name)
const icons = {
    "devicon-plain:bash": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M112.205 26.129L71.8 2.142A15.3 15.3 0 0 0 64.005 0c-2.688 0-5.386.717-7.796 2.152L15.795 26.14C10.976 28.999 8 34.289 8 40.018v47.975c0 5.729 2.967 11.019 7.796 13.878L56.2 125.858A15.2 15.2 0 0 0 63.995 128a15.3 15.3 0 0 0 7.796-2.142l40.414-23.987c4.819-2.86 7.796-8.16 7.796-13.878V40.007c0-5.718-2.967-11.019-7.796-13.878m-31.29 74.907l.063 3.448c0 .418-.267.889-.588 1.06l-2.046 1.178c-.321.16-.6-.032-.6-.45l-.032-3.394c-1.745.728-3.523.9-4.647.45c-.214-.086-.31-.397-.225-.761l.739-3.116c.064-.246.193-.493.364-.643a.7.7 0 0 1 .193-.139c.117-.064.235-.075.332-.032c1.22.407 2.773.214 4.272-.535c1.907-.964 3.18-2.913 3.16-4.84c-.022-1.757-.964-2.474-3.267-2.496c-2.934.01-5.675-.567-5.718-4.894c-.032-3.555 1.81-7.26 4.744-9.595l-.032-3.48c0-.428.257-.9.589-1.07l1.98-1.264c.322-.161.6.043.6.46l.033 3.48c1.456-.578 2.72-.738 3.865-.47c.247.064.364.406.257.802l-.77 3.084a1.4 1.4 0 0 1-.354.622a.8.8 0 0 1-.203.15c-.108.053-.204.064-.3.053c-.525-.118-1.767-.385-3.727.6c-2.056 1.038-2.773 2.827-2.763 4.155c.022 1.585.825 2.066 3.63 2.11c3.738.064 5.344 1.691 5.387 5.45c.053 3.684-1.917 7.657-4.937 10.077zm21.18-5.794c0 .322-.042.621-.31.771l-10.216 6.211c-.267.161-.482.022-.482-.3V99.29c0-.321.193-.492.46-.653l10.067-6.018c.268-.16.482-.022.482.3zm7.026-58.993L70.89 59.86c-4.765 2.784-8.278 5.911-8.288 11.662v47.107c0 3.437 1.392 5.665 3.523 6.318a13 13 0 0 1-2.12.204c-2.239 0-4.445-.61-6.383-1.757L17.219 99.408c-3.951-2.345-6.403-6.725-6.403-11.426V40.007c0-4.7 2.452-9.08 6.403-11.426L57.634 4.594a12.56 12.56 0 0 1 6.382-1.756c2.238 0 4.444.61 6.382 1.756l40.415 23.987c3.33 1.981 5.579 5.397 6.21 9.242c-1.36-2.86-4.38-3.63-7.902-1.574\"/></svg>",
    "devicon-plain:c": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"m117.5 33.5l.3-.2c-.6-1.1-1.5-2.1-2.4-2.6L67.1 2.9c-.8-.5-1.9-.7-3.1-.7s-2.3.3-3.1.7l-48 27.9c-1.7 1-2.9 3.5-2.9 5.4v55.7c0 1.1.2 2.3.9 3.4l-.2.1c.5.8 1.2 1.5 1.9 1.9l48.2 27.9c.8.5 1.9.7 3.1.7s2.3-.3 3.1-.7l48-27.9c1.7-1 2.9-3.5 2.9-5.4V36.1c.1-.8 0-1.7-.4-2.6M64 88.5c9.1 0 17.1-5 21.3-12.4l12.9 7.6c-6.8 11.8-19.6 19.8-34.2 19.8c-21.8 0-39.5-17.7-39.5-39.5S42.2 24.5 64 24.5c14.7 0 27.5 8.1 34.3 20l-13 7.5C81.1 44.5 73.1 39.5 64 39.5c-13.5 0-24.5 11-24.5 24.5s11 24.5 24.5 24.5\"/></svg>",
    "devicon-plain:cmake": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M62.8.4L.3 123.8l68.1-57.9zm61 127.3l-84-33.9L0 127.7zm4.2-1.1L65.6 2.5l9.2 102.6zM71.9 104l-3.1-34.9L42 92z\"/></svg>",
    "devicon-plain:cplusplus": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M63.443 0c-1.782 0-3.564.39-4.916 1.172L11.594 28.27C8.89 29.828 6.68 33.66 6.68 36.78v54.197c0 1.562.55 3.298 1.441 4.841l-.002.002c.89 1.543 2.123 2.89 3.475 3.672l46.931 27.094c2.703 1.562 7.13 1.562 9.832 0h.002l46.934-27.094c1.352-.78 2.582-2.129 3.473-3.672c.89-1.543 1.441-3.28 1.441-4.843V36.779c0-1.557-.55-3.295-1.441-4.838v-.002c-.891-1.545-2.121-2.893-3.473-3.67L68.359 1.173C67.008.39 65.226 0 63.443 0m.002 26.033c13.465 0 26.02 7.246 32.77 18.91l-16.38 9.479c-3.372-5.836-9.66-9.467-16.39-9.467c-10.432 0-18.922 8.49-18.922 18.924S53.013 82.8 63.445 82.8c6.735 0 13.015-3.625 16.395-9.465l16.375 9.477c-6.746 11.662-19.305 18.91-32.77 18.91c-20.867 0-37.843-16.977-37.843-37.844s16.976-37.844 37.843-37.844v-.002zM92.881 57.57h4.201v4.207h4.203v4.203h-4.203v4.207h-4.201V65.98h-4.207v-4.203h4.207zm15.765 0h4.208v4.207h4.203v4.203h-4.203v4.207h-4.208V65.98h-4.205v-4.203h4.205z\"/></svg>",
    "devicon-plain:csharp": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"m117.5 33.5l.3-.2c-.6-1.1-1.5-2.1-2.4-2.6L67.1 2.9c-.8-.5-1.9-.7-3.1-.7s-2.3.3-3.1.7l-48 27.9c-1.7 1-2.9 3.5-2.9 5.4v55.7c0 1.1.2 2.3.9 3.4l-.2.1c.5.8 1.2 1.5 1.9 1.9l48.2 27.9c.8.5 1.9.7 3.1.7s2.3-.3 3.1-.7l48-27.9c1.7-1 2.9-3.5 2.9-5.4V36.1c.1-.8 0-1.7-.4-2.6m-53.5 70c-21.8 0-39.5-17.7-39.5-39.5S42.2 24.5 64 24.5c14.7 0 27.5 8.1 34.3 20l-13 7.5C81.1 44.5 73.1 39.5 64 39.5c-13.5 0-24.5 11-24.5 24.5s11 24.5 24.5 24.5c9.1 0 17.1-5 21.3-12.4l12.9 7.6c-6.8 11.8-19.6 19.8-34.2 19.8M115 62h-3.2l-.9 4h4.1v5h-5l-1.2 6h-4.9l1.2-6h-3.8l-1.2 6h-4.8l1.2-6H94v-5h3.5l.9-4H94v-5h5.3l1.2-6h4.9l-1.2 6h3.8l1.2-6h4.8l-1.2 6h2.2zm-12.7 4h3.8l.9-4h-3.8z\"/></svg>",
    "devicon-plain:css3": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"m8.76 1l10.055 112.883l45.118 12.58l45.244-12.626L119.24 1zm89.591 25.862l-3.347 37.605l.01.203l-.014.467v-.004l-2.378 26.294l-.262 2.336L64 101.607v.001l-.022.019l-28.311-7.888L33.75 72h13.883l.985 11.054l15.386 4.17l-.004.008v-.002l15.443-4.229L81.075 65H48.792l-.277-3.043l-.631-7.129L47.553 51h34.749l1.264-14H30.64l-.277-3.041l-.63-7.131L29.401 23h69.281z\"/></svg>",
    "devicon-plain:dart": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><g fill=\"currentColor\"><path d=\"m86.6 25l3 .1c1.1.1 2.2.3 3.4.5l-2.5-7.4L75.7 3.5c-3.4-3.4-8-4.4-10.4-2.3L29.2 25.1zm6.1 3.6c-1.2-.2-2.3-.4-3.3-.5l-2.9-.1l-56 .1l78.6 78.6l6.1-13.8zM28.9 92.2l64.3 22.7l13.8-6.1l-78.6-78.6v56.1l.1 2.7c0 .9.1 2 .4 3.2\"/><path d=\"M106.9 34.3c-2.6-2.6-7-5.1-11.3-6.5L118.4 93l-6.9 15.7l15.8-5.2V54.8zm-13.5 83.8l-65-22.9c1.4 4.3 3.8 8.7 6.5 11.4l21.3 21.2l47.6.1l5.3-16.7zm-67.9-29l-.1-2.7V28.9L1.7 65.1C-.4 67.3.7 72 4 75.5l14.7 14.8l7.3 2.6c-.3-1.3-.5-2.5-.5-3.8\"/></g></svg>",
    "devicon-plain:docker": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M124.8 52.1c-4.3-2.5-10-2.8-14.8-1.4c-.6-5.2-4-9.7-8-12.9l-1.6-1.3l-1.4 1.6c-2.7 3.1-3.5 8.3-3.1 12.3c.3 2.9 1.2 5.9 3 8.3c-1.4.8-2.9 1.9-4.3 2.4c-2.8 1-5.9 2-8.9 2H79V49H66V24H51v12H26v13H13v14H1.8l-.2 1.5c-.5 6.4.3 12.6 3 18.5l1.1 2.2l.1.2c7.9 13.4 21.7 19 36.8 19c29.2 0 53.3-13.1 64.3-40.6c7.4.4 15-1.8 18.6-8.9l.9-1.8zM28 39h10v11H28zm13.1 44.2c0 1.7-1.4 3.1-3.1 3.1s-3.1-1.4-3.1-3.1s1.4-3.1 3.1-3.1c1.7.1 3.1 1.4 3.1 3.1M28 52h10v11H28zm-13 0h11v11H15zm27.7 50.2c-15.8-.1-24.3-5.4-31.3-12.4c2.1.1 4.1.2 5.9.2c1.6 0 3.2 0 4.7-.1c3.9-.2 7.3-.7 10.1-1.5c2.3 5.3 6.5 10.2 14 13.8zM51 63H40V52h11zm0-13H40V39h11zm13 13H53V52h11zm0-13H53V39h11zm0-13H53V26h11zm13 26H66V52h11zM38.8 81.2c-.2-.1-.5-.2-.8-.2c-1.2 0-2.2 1-2.2 2.2s1 2.2 2.2 2.2s2.2-1 2.2-2.2c0-.3-.1-.6-.2-.8c-.2.3-.4.5-.8.5c-.5 0-.9-.4-.9-.9c.1-.4.3-.7.5-.8\"/></svg>",
    "devicon-plain:elixir": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M90.1 80c.8-.5 1.7-1 2.5-1.5c5.6-4.8 3.6-13.5-5.9-25.7c.2 6.1 1.2 12.4 2.2 19.1q.6 3.9 1.2 8.1M36 103.2c3.2 13.7 11.3 21.1 20.5 24.2c5.1.9 10.2.7 15.3-.4c.8-.4 1.5-.9 2.2-1.4c2-1.3 3.7-2.8 5.3-4.3c-6.5-2.8-13.1-8.2-19.3-14.6c-4.8-.3-9.5-.8-13.6-1.4c-4-.4-7.4-1.1-10.4-2.1m-1.4-64.5l-.2 1.4c-1.4 7.9-1.5 15-.4 21.2c1.1-7.4 2.6-15 4.7-22.3c.3-1.1.6-2.3 1-3.5h.1c1.6-5 3.4-9.8 5.5-14.3c-4 5.3-7.5 11.2-10.7 17.5\"/><path fill=\"currentColor\" d=\"M33.9 114c3.5 4.4 7.5 7.6 11.9 9.9c-5.6-4.6-10-11.5-12-21.4c-3.8-1.4-6.8-3.3-9.1-5.7c1.6 6.4 4.6 12.3 9.2 17.2m-.6-47q-2.7-8.85-1.8-20.4c-1.1 2.3-2.1 4.8-3.2 7.3c-4.5 13.1-6.5 26.7-4.6 38.7c2.1 3.2 5.2 5.8 9.6 7.6c-1.4-7.7-1.5-19.9 0-33.2m2.2 33.8q0 .15 0 0c3.2 1.2 6.8 2 11.1 2.5c3.5.6 7.3 1 11.3 1.3c-8.3-9.1-15.6-19.8-20.3-28.6c-1.1-1.6-2-3.3-2.8-5c-1.1 12.2-.8 23.1.7 29.8m51.4-28.7c-1.1-7.5-2.3-15.1-2.3-22.3C71.9 38 59.6 25.4 60.1 5.1c-.4.3-.8.6-1.1.9c-2.2 1.9-4.4 4-6.5 6.3c-4.5 6.4-8.1 14.6-10.9 23.5c-1.6 11.6 9.6 34.3 24.5 51.7c7.2-.3 14.8-2.5 22.1-6.5c-.4-2.9-.8-5.9-1.3-8.9m10 39.7c-2.6-.5-5.2-1.4-7.8-2.6c-1.3 4.1-3.4 7.9-6.4 11.4c1.1.3 2.1.6 3.1.7c4.2-2.5 8-5.8 11.1-9.5\"/><path fill=\"currentColor\" d=\"M62.1 3.6C61 23.1 72.2 35.5 84.4 46.8C73.7 32.6 67.5 24.1 67.5.3q-2.55 1.35-5.4 3.3m21.1 102.2c-5.3.8-10.8 1.1-16.2 1.1c-1.4 0-2.9 0-4.3-.1c5.8 5.8 12 10.6 18 13c3.2-3.5 5.3-7.4 6.6-11.6c-1.4-.7-2.8-1.5-4.1-2.4m-2.4-1.6c-2-1.4-3.8-2.9-5.5-4.4c-3.5-3.1-6.9-6.5-10.1-10.2c-7.1.1-13.7-1.6-19.1-5.1c-.8-.5-1.6-1.1-2.3-1.7c4.6 7.3 10.5 15.3 16.9 22c6.5.3 13.5.2 20.1-.6m9.6.2c-.2 1-.4 1.9-.7 2.9c2.8 1.4 5.7 2.4 8.6 2.7c2.5-3.4 4.5-7.2 5.7-11.3c-3.7 2.5-8.4 4.4-13.6 5.7\"/><path fill=\"currentColor\" d=\"M47.2 82.8c4.7 3 10.3 4.6 16.3 4.8c-12.3-14.6-21.8-32.4-23.7-45c-2.3 8.9-3.1 14.4-4.4 23.6v.2c.9 2.3 2.1 5.1 3.7 8.1c2.1 3.4 4.8 6.2 8.1 8.3m41.3.4c-6.8 3.7-13.9 5.8-20.7 6.3c2.9 3.2 5.8 6.2 8.8 8.8c2.1 1.9 4.5 3.8 7.1 5.5c1.7-.3 3.3-.6 4.9-.9c1-6.3.7-12.9-.1-19.7m2.2 19.1c5.6-1.5 10.5-3.6 14.1-6.6c1.5-6.9 1-14.4-2.3-22.4c-2.9-4.9-5.7-9.2-8.3-13c4.8 8.8 4.7 15.2-.4 19.6l-.1.1c-1.1.7-2.2 1.4-3.4 2.1c.9 6.9 1.3 13.8.4 20.2\"/></svg>",
    "devicon-plain:elm": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"m64 60.74l25.65-25.65h-51.3zM7.91 4.65l25.83 25.84h56.17L64.07 4.65zm59.353 59.343l28.08-28.08l27.951 27.953l-28.08 28.079zm56.087-6.573V4.65H70.58zM60.74 64L4.65 7.91V120.1zm37.73 31.21l24.88 24.89V70.33zM64 67.26L7.91 123.35h112.18z\"/></svg>",
    "devicon-plain:erlang": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M18.2 24.1L1 24v80h19.7v-.1C11 93.6 5.2 79.2 5.3 62.1C5.2 47 10 33.9 18.2 24.1M127 24h-16.4c6.2 9 9.6 19.3 9.1 32.1c.1 1.2.1 1.9 0 4.9H46.3c0 22 7.7 38.3 27.3 38.4c13.5-.1 23.2-10.1 29.9-20.9l19 9.5c-3.4 6.1-7.2 11-11.4 16H127zm-61.9 1.6c-9 0-16.8 7.4-17.6 16.4H81c-.3-9-6.8-16.4-15.9-16.4\"/></svg>",
    "devicon-plain:fsharp": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M0 64.5L60.7 3.8v30.4L30.4 64.5l30.4 30.4v30.4zm39.1 0l21.7-21.7v43.4zm88.9 0L65.1 3.8v30.4l30.4 30.4l-30.4 30.3v30.4z\"/></svg>",
    "devicon-plain:gleam": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M56.61.014q-.616.032-1.221.138v.002c-3.23.57-6.182 2.714-7.405 6.178l-9.523 26.98a9.6 9.6 0 0 1-5.553 5.745L6.244 49.504C-.598 52.184-1.923 61.596 3.91 66.05l22.746 17.37a9.58 9.58 0 0 1 3.75 7.047l1.696 28.553c.434 7.326 8.978 11.493 15.033 7.323v-.002l23.58-16.244a9.6 9.6 0 0 1 7.87-1.389l27.714 7.2c7.116 1.848 13.72-4.99 11.623-12.022l-8.17-27.412a9.58 9.58 0 0 1 1.113-7.905l15.432-24.103c3.958-6.182-.503-14.573-7.85-14.752l-28.63-.695a9.6 9.6 0 0 1-7.182-3.499L64.459 3.426C62.418.944 59.477-.137 56.609.014zm30.435 55.513a5.082 5.082 0 1 1 .222 10.161a5.082 5.082 0 0 1-.222-10.16zM46.1 62.747a5.083 5.083 0 0 1 .992 10.085a5.082 5.082 0 1 1-.992-10.086zm26.283 7.382a2.6 2.6 0 0 1 1.812.799a2.6 2.6 0 0 1 .713 1.847a6.72 6.72 0 0 1-2.088 4.721h-.002a6.7 6.7 0 0 1-2.22 1.41h-.002a6.8 6.8 0 0 1-2.59.451h-.002a6.7 6.7 0 0 1-2.567-.576h-.001a6.76 6.76 0 0 1-3.563-3.736a2.59 2.59 0 0 1 .63-2.805a2.6 2.6 0 0 1 1.847-.715a2.595 2.595 0 0 1 2.35 1.655c.075.192.186.368.328.517v.002q.216.224.5.35v.002h.002a1.58 1.58 0 0 0 1.201.03l.002-.003a1.55 1.55 0 0 0 .868-.828h.002v-.002c.083-.188.127-.389.132-.594a2.59 2.59 0 0 1 1.653-2.353c.317-.122.654-.18.994-.172z\"/></svg>",
    "devicon-plain:go": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M108.2 64.8c-.1-.1-.2-.2-.4-.2l-.1-.1c-.1-.1-.2-.1-.2-.2l-.1-.1c-.1 0-.2-.1-.2-.1l-.2-.1c-.1 0-.2-.1-.2-.1l-.2-.1c-.1 0-.2-.1-.2-.1c-.1 0-.1 0-.2-.1l-.3-.1c-.1 0-.1 0-.2-.1l-.3-.1h-.1l-.4-.1h-.2c-.1 0-.2 0-.3-.1h-2.3c-.6-13.3.6-26.8-2.8-39.6c12.9-4.6 2.8-22.3-8.4-14.4c-7.4-6.4-17.6-7.8-28.3-7.8c-10.5.7-20.4 2.9-27.4 8.4c-2.8-1.4-5.5-1.8-7.9-1.1v.1c-.1 0-.3.1-.4.2c-.1 0-.3.1-.4.2h-.1c-.1 0-.2.1-.4.2h-.1l-.3.2h-.1l-.3.2h-.1l-.3.2s-.1 0-.1.1l-.3.2s-.1 0-.1.1l-.3.2s-.1 0-.1.1l-.3.2l-.1.1c-.1.1-.2.1-.2.2l-.1.1l-.2.2l-.1.1c-.1.1-.1.2-.2.2l-.1.1c-.1.1-.1.2-.2.2l-.1.1c-.1.1-.1.2-.2.2l-.1.1c-.1.1-.1.2-.2.2l-.1.1c-.1.1-.1.2-.2.2l-.1.1l-.1.3s0 .1-.1.1l-.1.3s0 .1-.1.1l-.1.3s0 .1-.1.1l-.1.3s0 .1-.1.1c.4.3.4.4.4.4v.1l-.1.3v.1c0 .1 0 .2-.1.3v3.1c0 .1 0 .2.1.3v.1l.1.3v.1l.1.3s0 .1.1.1l.1.3s0 .1.1.1l.1.3s0 .1.1.1l.2.3s0 .1.1.1l.2.3s0 .1.1.1l.2.3l.1.1l.3.3l.3.3h.1c1 .9 2 1.6 4 2.2v-.2C23 37.3 26.5 50 26.7 63c-.6 0-.7.4-1.7.5h-.5c-.1 0-.3 0-.5.1c-.1 0-.3 0-.4.1l-.4.1h-.1l-.4.1h-.1l-.3.1h-.1l-.3.1s-.1 0-.1.1l-.3.1l-.2.1c-.1 0-.2.1-.2.1l-.2.1l-.2.1c-.1 0-.2.1-.2.1l-.2.1l-.4.3c-.1.1-.2.2-.3.2l-.4.4l-.1.1c-.1.2-.3.4-.4.5l-.2.3l-.3.6l-.1.3v.3c0 .5.2.9.9 1.2c.2 3.7 3.9 2 5.6.8l.1-.1c.2-.2.5-.3.6-.3h.1l.2-.1c.1 0 .1 0 .2-.1c.2-.1.4-.1.5-.2c.1 0 .1-.1.1-.2l.1-.1c.1-.2.2-.6.2-1.2l.1-1.3v1.8c-.5 13.1-4 30.7 3.3 42.5c1.3 2.1 2.9 3.9 4.7 5.4h-.5c-.2.2-.5.4-.8.6l-.9.6l-.3.2l-.6.4l-.9.7l-1.1 1c-.2.2-.3.4-.4.5l-.4.6l-.2.3c-.1.2-.2.4-.2.6l-.1.3q-.3 1.2.6 2.7l.4.4h.2c.1 0 .2 0 .4.1c.2.4 1.2 2.5 3.9.9c2.8-1.5 4.7-4.6 8.1-5.1l-.5-.6c5.9 2.8 12.8 4 19 4.2c8.7.3 18.6-.9 26.5-5.2c2.2.7 3.9 3.9 5.8 5.4l.1.1l.1.1l.1.1l.1.1s.1 0 .1.1c0 0 .1 0 .1.1c0 0 .1 0 .1.1h2.1s.1 0 .1-.1h.1s.1 0 .1-.1h.1s.1 0 .1-.1c0 0 .1 0 .1-.1l.1-.1s.1 0 .1-.1l.1-.1h.1l.2-.2l.2-.1h.1l.1-.1h.1l.1-.1l.1-.1l.1-.1l.1-.1l.1-.1l.1-.1l.1-.1v-.1s0-.1.1-.1v-.1s0-.1.1-.1v-.1s0-.1.1-.1v-1.4s-.3 0-.3-.1l-.3-.1v-.1l.3-.1s.2 0 .2-.1l.1-.1v-2.1s0-.1-.1-.1v-.1s0-.1-.1-.1v-.1s0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1c0 0 0-.1-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1v-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1l-.1-.1c2-1.9 3.8-4.2 5.1-6.9c5.9-11.8 4.9-26.2 4.1-39.2h.1c.1 0 .2.1.2.1h.3s.1 0 .1.1h.1s.1 0 .1.1l.2.1c1.7 1.2 5.4 2.9 5.6-.8c1.6.6-.3-1.8-1.3-2.5M36 23C32.8 7 58.4 4 59.3 19.6c.8 13-20 16.3-23.3 3.4m36.1 15c-1.3 1.4-2.7 1.2-4.1.7c0 1.9.4 3.9.1 5.9c-.5.9-1.5 1-2.3 1.4c-1.2-.2-2.1-.9-2.6-2l-.2-.1c-3.9 5.2-6.3-1.1-5.2-5c-1.2.1-2.2-.2-3-1.5c-1.4-2.6.7-5.8 3.4-6.3c.7 3 8.7 2.6 10.1-.2c3.1 1.5 6.5 4.3 3.8 7.1m-7-17.5c-.9-13.8 20.3-17.5 23.4-4c3.5 15-20.8 18.9-23.4 4M41.7 17c-1.9 0-3.5 1.7-3.5 3.8s1.6 3.8 3.5 3.8s3.5-1.7 3.5-3.8s-1.5-3.8-3.5-3.8m1.6 5.7c-.5 0-.8-.4-.8-1c0-.5.4-1 .8-1c.5 0 .8.4.8 1c0 .5-.3 1-.8 1m27.8-6.6c-1.9 0-3.4 1.7-3.4 3.8s1.5 3.8 3.4 3.8s3.4-1.7 3.4-3.8s-1.5-3.8-3.4-3.8m1.6 5.6c-.4 0-.8-.4-.8-1c0-.5.4-1 .8-1s.8.4.8 1s-.4 1-.8 1\"/></svg>",
    "devicon-plain:graphql": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><g fill=\"currentColor\"><path d=\"m18.39 96.852l-4.6-2.657L65.04 5.434l4.597 2.656zm0 0\"/><path d=\"M12.734 87.105H115.23v5.31H12.734zm0 0\"/><path d=\"M66.031 119.688L14.766 90.09l2.656-4.602l51.266 29.602zm44.535-77.145L59.301 12.941l2.656-4.597l51.266 29.597zm0 0\"/><path d=\"m17.434 42.523l-2.657-4.601l51.27-29.598l2.656 4.598zm0 0\"/><path d=\"M109.621 96.852L58.375 8.09l4.598-2.656l51.25 88.761zM16.8 34.398h5.313v59.204h-5.312zm0 0\"/><path d=\"M105.887 34.398h5.312v59.204h-5.312zm0 0\"/><path d=\"m65.129 117.441l-2.32-4.02l44.586-25.745l2.32 4.02zm0 0\"/><path d=\"M118.238 95.328c-3.07 5.344-9.918 7.168-15.261 4.098c-5.344-3.074-7.168-9.922-4.098-15.266c3.074-5.344 9.922-7.168 15.266-4.097c5.375 3.105 7.199 9.921 4.093 15.265M29.09 43.84c-3.074 5.344-9.922 7.168-15.266 4.097c-5.344-3.074-7.168-9.921-4.097-15.265c3.074-5.344 9.921-7.168 15.265-4.098c5.344 3.106 7.168 9.922 4.098 15.266M9.762 95.328c-3.075-5.344-1.25-12.16 4.093-15.266c5.344-3.07 12.16-1.246 15.266 4.098c3.07 5.344 1.246 12.16-4.098 15.266c-5.375 3.07-12.191 1.246-15.261-4.098M98.91 43.84c-3.07-5.344-1.246-12.16 4.098-15.266c5.344-3.07 12.16-1.246 15.265 4.098c3.07 5.344 1.247 12.16-4.097 15.266c-5.344 3.07-12.192 1.246-15.266-4.098M64 126.656a11.16 11.16 0 0 1-11.168-11.168A11.16 11.16 0 0 1 64 104.32a11.16 11.16 0 0 1 11.168 11.168c0 6.145-4.992 11.168-11.168 11.168M64 23.68a11.16 11.16 0 0 1-11.168-11.168A11.16 11.16 0 0 1 64 1.344a11.16 11.16 0 0 1 11.168 11.168A11.16 11.16 0 0 1 64 23.68\"/></g></svg>",
    "devicon-plain:haskell": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M0 110.2L30.1 65L0 19.9h22.6L52.7 65l-30.1 45.1H0z\"/><path fill=\"currentColor\" d=\"M30.1 110.2L60.2 65L30.1 19.9h22.6l60.2 90.3H90.4L71.5 81.9l-18.8 28.2H30.1zm72.8-26.4l-10-15.1H128v15.1zM87.8 61.3l-10-15.1H128v15.1z\"/></svg>",
    "devicon-plain:html5": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"m9.032 2l10.005 112.093l44.896 12.401l45.02-12.387L118.968 2zm89.126 26.539l-.627 7.172L97.255 39H44.59l1.257 14h50.156l-.336 3.471l-3.233 36.119l-.238 2.27L64 102.609v.002l-.034.018l-28.177-7.423L33.876 74h13.815l.979 10.919L63.957 89H64v-.546l15.355-3.875L80.959 67H33.261l-3.383-38.117L29.549 25h68.939z\"/></svg>",
    "devicon-plain:java": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M47.617 98.12c-19.192 5.362 11.677 16.439 36.115 5.969c-4.003-1.556-6.874-3.351-6.874-3.351c-10.897 2.06-15.952 2.222-25.844 1.092c-8.164-.935-3.397-3.71-3.397-3.71m33.189-10.46c-14.444 2.779-22.787 2.69-33.354 1.6c-8.171-.845-2.822-4.805-2.822-4.805c-21.137 7.016 11.767 14.977 41.309 6.336c-3.14-1.106-5.133-3.131-5.133-3.131m11.319-60.575c.001 0-42.731 10.669-22.323 34.187c6.024 6.935-1.58 13.17-1.58 13.17s15.289-7.891 8.269-17.777c-6.559-9.215-11.587-13.793 15.634-29.58m9.998 81.144s3.529 2.91-3.888 5.159c-14.102 4.272-58.706 5.56-71.095.171c-4.45-1.938 3.899-4.625 6.526-5.192c2.739-.593 4.303-.485 4.303-.485c-4.952-3.487-32.013 6.85-13.742 9.815c49.821 8.076 90.817-3.637 77.896-9.468M85 77.896c2.395-1.634 5.703-3.053 5.703-3.053s-9.424 1.685-18.813 2.474c-11.494.964-23.823 1.154-30.012.326c-14.652-1.959 8.033-7.348 8.033-7.348s-8.812-.596-19.644 4.644C17.455 81.134 61.958 83.958 85 77.896m5.609 15.145c-.108.29-.468.616-.468.616c31.273-8.221 19.775-28.979 4.822-23.725c-1.312.464-2 1.543-2 1.543s.829-.334 2.678-.72c7.559-1.575 18.389 10.119-5.032 22.286M64.181 70.069c-4.614-10.429-20.26-19.553.007-35.559C89.459 14.563 76.492 1.587 76.492 1.587c5.23 20.608-18.451 26.833-26.999 39.667c-5.821 8.745 2.857 18.142 14.688 28.815m27.274 51.748c-19.187 3.612-42.854 3.191-56.887.874c0 0 2.874 2.38 17.646 3.331c22.476 1.437 57-.8 57.816-11.436c.001 0-1.57 4.032-18.575 7.231\"/></svg>",
    "devicon-plain:javascript": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M2 1v125h125V1zm66.119 106.513c-1.845 3.749-5.367 6.212-9.448 7.401c-6.271 1.44-12.269.619-16.731-2.059c-2.986-1.832-5.318-4.652-6.901-7.901l9.52-5.83c.083.035.333.487.667 1.071c1.214 2.034 2.261 3.474 4.319 4.485c2.022.69 6.461 1.131 8.175-2.427c1.047-1.81.714-7.628.714-14.065C58.433 78.073 58.48 68 58.48 58h11.709c0 11 .06 21.418 0 32.152c.025 6.58.596 12.446-2.07 17.361m48.574-3.308c-4.07 13.922-26.762 14.374-35.83 5.176c-1.916-2.165-3.117-3.296-4.26-5.795c4.819-2.772 4.819-2.772 9.508-5.485c2.547 3.915 4.902 6.068 9.139 6.949c5.748.702 11.531-1.273 10.234-7.378c-1.333-4.986-11.77-6.199-18.873-11.531c-7.211-4.843-8.901-16.611-2.975-23.335c1.975-2.487 5.343-4.343 8.877-5.235l3.688-.477c7.081-.143 11.507 1.727 14.756 5.355c.904.916 1.642 1.904 3.022 4.045c-3.772 2.404-3.76 2.381-9.163 5.879c-1.154-2.486-3.069-4.046-5.093-4.724c-3.142-.952-7.104.083-7.926 3.403c-.285 1.023-.226 1.975.227 3.665c1.273 2.903 5.545 4.165 9.377 5.926c11.031 4.474 14.756 9.271 15.672 14.981c.882 4.916-.213 8.105-.38 8.581\"/></svg>",
    "devicon-plain:julia": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><circle cx=\"29.1\" cy=\"94.2\" r=\"29.1\" fill=\"currentColor\"/><circle cx=\"98.9\" cy=\"94.2\" r=\"29.1\" fill=\"currentColor\"/><circle cx=\"64\" cy=\"33.8\" r=\"29.1\" fill=\"currentColor\"/></svg>",
    "devicon-plain:kotlin": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M112.484 112.484H15.516V15.516h96.968L64 64Zm0 0\"/></svg>",
    "devicon-plain:lua": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M61.7 0c-1.9 0-3.8.2-5.6.4l.2 1.5c1.8-.2 3.6-.4 5.5-.4zm5.6 0l-.1 1.5c1.8.1 3.6.3 5.4.5l.3-1.5C71 .3 69.2.1 67.3 0m45.7.8c-7.9 0-14.4 6.3-14.4 14.3S105 29.4 113 29.4s14.3-6.4 14.3-14.3S120.9.8 113 .8m-62.4.6c-1.8.4-3.6.9-5.4 1.4l.4 1.4c1.7-.5 3.5-1 5.3-1.4zm27.6.3l-.3 1.4c1.8.4 3.6.8 5.3 1.4l.4-1.3q-2.7-.9-5.4-1.5m-38.3 3c-1.7.7-3.4 1.5-5.1 2.3l.7 1.3c1.6-.8 3.3-1.6 5-2.3zm49 .3l-.5 1.4c1.6.7 3.3 1.5 4.9 2.3l.6-1.3c-1.6-.9-3.3-1.7-5-2.4M30 9.7c-1.6 1-3.1 2.1-4.6 3.2l.9 1.2c1.4-1.1 2.9-2.1 4.5-3.2zm34 5.4c-27 0-49 21.9-49 49s21.9 49 49 49s49-21.9 49-49s-22-49-49-49m-42.9 1.4c-1.4 1.2-2.7 2.5-4 3.9l1.1 1c1.2-1.3 2.5-2.6 3.9-3.8zm-7.6 8.2c-1.1 1.4-2.2 2.9-3.2 4.5l1.2.8c1-1.5 2-3 3.2-4.4zm70.8 4.7c7.9 0 14.3 6.4 14.3 14.3S92.2 58 84.3 58S70 51.6 70 43.7s6.4-14.3 14.3-14.3M7.4 34.1c-.9 1.6-1.7 3.3-2.4 5l1.4.5c.7-1.6 1.5-3.3 2.3-4.8zm113.6.8l-1.3.7c.9 1.6 1.6 3.3 2.3 5l1.3-.6c-.7-1.7-1.5-3.4-2.3-5.1M3.1 44.3q-.9 2.7-1.5 5.4L3 50c.4-1.8.9-3.5 1.5-5.2zm122.1 1l-1.4.4c.6 1.7 1 3.5 1.4 5.3l1.4-.3c-.4-1.8-.9-3.6-1.4-5.4M.5 55.1C.3 57 .1 58.8 0 60.7l1.5.1c.1-1.8.3-3.6.5-5.4zm127.1 1.1l-1.5.2c.2 1.8.3 3.6.4 5.4h1.5c0-1.9-.2-3.8-.4-5.6m-96.9.2h4.1v28.5h15.9v3.6h-20zm57.7 8.3c5.7 0 8.7 2.2 8.7 6.3v13.6c0 1.1.7 1.8 2 1.8c.2 0 .4 0 .8-.1l-.1 2.8c-1.2.3-1.8.4-2.5.4c-2.4 0-3.5-1.1-3.8-3.4c-2.6 2.4-4.9 3.4-7.8 3.4c-4.7 0-7.6-2.6-7.6-6.8c0-3 1.4-5.1 4.1-6.2c1.4-.6 2.2-.7 7.4-1.4c2.9-.4 3.8-1 3.8-2.6v-1c0-2.2-1.9-3.4-5.2-3.4c-3.4 0-5.1 1.3-5.4 4.1h-3.7c.1-2.3.5-3.6 1.6-4.8c1.5-1.7 4.3-2.7 7.7-2.7m-33.8.7h3.7v16.3c0 2.8 1.9 4.5 4.8 4.5c3.8 0 6.3-3.1 6.3-7.8v-13H73v23.1h-3.3v-3.2c-2.2 3-4.3 4.2-7.7 4.2c-4.5 0-7.4-2.5-7.4-6.3zm-53.1.8l-1.5.1c.1 1.9.2 3.7.5 5.6l1.4-.3c-.2-1.8-.3-3.6-.4-5.4m124.9 1.1c-.1 1.8-.3 3.6-.5 5.4l1.5.2c.3-1.8.5-3.7.5-5.5zM2.8 77.1l-1.4.3c.4 1.8.9 3.6 1.4 5.4l1.4-.4c-.6-1.8-1-3.5-1.4-5.3m90.6 0c-1.2.6-2 .7-5.9 1.3c-3.9.5-5.6 1.8-5.6 4.2c0 2.3 1.7 3.7 4.5 3.7c2.2 0 4-.7 5.5-2.1c1.1-1 1.5-1.8 1.5-3zm31.6.9c-.5 1.8-.9 3.6-1.5 5.3l1.4.5c.6-1.8 1.1-3.6 1.5-5.5zM6 87.5l-1.3.5c.7 1.7 1.5 3.4 2.3 5.1l1.3-.6c-.9-1.7-1.6-3.3-2.3-5m115.7 1c-.8 1.6-1.5 3.3-2.4 4.9l1.3.7L123 89zM10.9 97.2l-1.2.8c1 1.6 2.1 3.1 3.2 4.6l1.1-.9c-1.1-1.5-2.1-3-3.1-4.5m105.6.9c-1 1.5-2.1 3-3.2 4.4l1.2.9c1.1-1.4 2.2-3 3.2-4.5zm-98.9 7.8l-1.1 1c1.2 1.4 2.5 2.7 3.9 4l1-1.1c-1.3-1.2-2.6-2.6-3.8-3.9m92.2.8c-1.2 1.3-2.6 2.6-3.9 3.8l1 1.1c1.3-1.3 2.7-2.6 4-3.9zm-84.2 6.6l-.9 1.2c1.4 1.1 2.9 2.2 4.5 3.2l.8-1.2c-1.5-1-3-2.1-4.4-3.2m76.1.7c-1.5 1.1-3 2.1-4.5 3.1l.8 1.2c1.5-1 3.1-2 4.6-3.1zm-67 5.3l-.7 1.3c1.6.9 3.3 1.7 5 2.4l.6-1.3c-1.6-.8-3.3-1.5-4.9-2.4m57.7.4c-1.7.9-3.3 1.6-5 2.3l.6 1.4q2.55-1.05 5.1-2.4zm-47.7 3.8l-.5 1.4q2.7.9 5.4 1.5l.4-1.4c-1.8-.5-3.6-.9-5.3-1.5m37.6.3c-1.8.6-3.5 1-5.3 1.4l.3 1.4c1.9-.3 3.7-.8 5.4-1.4zm-27 2.2l-.2 1.5c1.9.2 3.7.4 5.6.5v-1.5c-1.8-.1-3.6-.3-5.4-.5m16.3.1c-1.8.2-3.6.3-5.4.4l.1 1.5c1.8-.1 3.7-.2 5.5-.4z\"/></svg>",
    "devicon-plain:matlab": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M123.965 91.902c-7.246-18.297-13.262-37.058-20.184-55.476c-3.054-7.84-6.047-15.746-10.215-23.082c-1.656-2.633-3.238-5.528-5.953-7.215a4 4 0 0 0-2.222-.606c-1.27.028-2.536.594-3.504 1.415c-3.645 2.886-5.805 7.082-8.227 10.949c-4.277 7.172-8.789 14.687-15.941 19.347c-3.36 2.371-7.762 2.63-11 5.172c-4.43 3.34-7.442 8.078-11.074 12.184c-.829.988-2.11 1.383-3.227 1.918C21.578 60.93 10.738 65.336 0 69.98c9.09 7.032 18.777 13.29 28.05 20.079c2.544-.504 5.098-1.547 7.72-1.082c4.16 1.3 6.597 5.285 8.503 8.93c3.875 7.94 6.676 16.323 9.813 24.57c5.246-.375 9.969-3.079 14.027-6.258c7.809-6.324 13.758-14.5 20.305-22.047c3.14-3.3 6.34-7.23 11.05-8.149c4.762-1.152 9.864.555 13.395 3.836c4.957 4.43 9.344 9.551 15.137 12.942c-.777-3.836-2.645-7.278-4.035-10.899M42.96 79.012c-4.57 2.703-9.426 4.93-14.176 7.289c-7.457-4.996-14.723-10.29-22.05-15.465c9.878-4.328 19.91-8.348 29.917-12.387c4.746 3.703 9.637 7.223 14.383 10.926c-2.23 3.563-4.914 6.871-8.074 9.637m10.168-12.414C48.414 63.058 43.64 59.609 39 55.977c2.977-4.055 6.238-7.977 10.14-11.172c2.587-1.657 5.743-2.117 8.426-3.61c6.368-3.18 10.711-9.011 14.86-14.582c-5.317 13.805-10.992 27.664-19.297 39.985zm0 0\"/></svg>",
    "devicon-plain:nixos": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" fill-rule=\"evenodd\" d=\"m39.831 65.463l30.202 52.66l-13.88.131l-8.063-14.148l-8.12 14.072l-6.897-.002l-3.532-6.143l11.57-20.024l-8.213-14.386zm10.901-21.692L20.525 96.43l-7.052-12.034l8.14-14.103l-16.167-.042L2 64.237l3.519-6.15l23.013.073l8.27-14.352zm2.318 42.094l60.409.003l-6.827 12.164l-16.205-.045l8.047 14.115l-3.45 6.01l-7.05.008l-11.445-20.097l-16.483-.034zm35.16-23.074l-30.202-52.66L71.888 10l8.063 14.148l8.12-14.072l6.897.002l3.532 6.143l-11.57 20.024l8.213 14.386z\" clip-rule=\"evenodd\"/><path fill=\"currentColor\" fill-rule=\"evenodd\" d=\"m39.831 65.463l30.202 52.66l-13.88.131l-8.063-14.148l-8.12 14.072l-6.897-.002l-3.532-6.143l11.57-20.024l-8.213-14.386zm35.08-23.207l-60.409-.003L21.33 30.09l16.204.045l-8.047-14.115l3.45-6.01l7.051-.01l11.444 20.097l16.484.034zm2.357 42.216l30.207-52.658l7.052 12.034l-8.141 14.102l16.168.043L126 64.006l-3.519 6.15l-23.013-.073l-8.27 14.352z\" clip-rule=\"evenodd\"/></svg>",
    "devicon-plain:objectivec": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><g fill=\"currentColor\"><path d=\"M63.877 125.392c-32.671 0-60.37-27.594-60.627-60.469a59.94 59.94 0 0 1 17.506-42.759a60.94 60.94 0 0 1 43.279-18.36a60.08 60.08 0 0 1 42.647 17.71a60.15 60.15 0 0 1 18.157 42.522c.151 33.604-26.864 61.021-60.469 61.363h-.493zm.19-118.406a57.77 57.77 0 0 0-41.01 17.427a56.78 56.78 0 0 0-16.63 40.484c.236 31.159 26.495 57.286 57.43 57.286h.414c31.863-.29 57.504-26.266 57.385-58.128a56.97 56.97 0 0 0-17.217-40.273A56.7 56.7 0 0 0 64.068 6.986z\"/><path d=\"M16.89 82.383V46.865h8.64v3.183h-4.583v29.218h4.584v3.183zm29.323-18.111c0 6.478-3.933 10.167-9.26 10.167s-8.877-4.156-8.877-9.831c0-5.939 3.722-10.121 9.167-10.121s8.97 4.36 8.97 9.785m-14.415.29c0 3.932 1.973 7.05 5.36 7.05s5.333-3.183 5.333-7.195c0-3.643-1.796-7.083-5.334-7.083s-5.392 3.328-5.392 7.307zm17.407-9.404c1.69-.29 3.407-.434 5.123-.428a9.17 9.17 0 0 1 5.537 1.223a4.06 4.06 0 0 1 2.006 3.61a4.48 4.48 0 0 1-3.183 4.183c2.269.46 3.9 2.46 3.9 4.775a5.02 5.02 0 0 1-1.861 3.978c-1.368 1.21-3.643 1.796-7.162 1.796a34 34 0 0 1-4.327-.257zm3.499 7.622h1.795c2.433 0 3.801-1.145 3.801-2.782s-1.368-2.644-3.61-2.644a9.8 9.8 0 0 0-2.006.145zm0 8.878c.618.065 1.243.092 1.86.078c2.263 0 4.262-.861 4.262-3.182s-1.94-3.183-4.373-3.183h-1.75zM69.54 54.901h3.517v12.554c0 5.334-2.577 7.116-6.365 7.116a9.3 9.3 0 0 1-2.973-.507l.428-2.834c.703.224 1.44.335 2.183.349c2.006 0 3.183-.921 3.183-4.262l.026-12.416zm13.527 10.456v2.434h-7.32v-2.434zm17.091 8.273a12.3 12.3 0 0 1-4.978.862c-6.129 0-9.851-3.834-9.851-9.707c-.283-5.353 3.827-9.923 9.18-10.206c.375-.02.757-.02 1.131.006a11.1 11.1 0 0 1 4.775.862l-.783 2.801a9.5 9.5 0 0 0-3.788-.75c-3.932 0-6.76 2.467-6.76 7.116c0 4.235 2.499 6.971 6.734 6.971a9.8 9.8 0 0 0 3.834-.717zM111.2 46.766v35.61h-8.641v-3.182h4.583V49.949h-4.583v-3.183h8.64z\"/></g></svg>",
    "devicon-plain:ocaml": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M65.004 115.355c-.461-.894-1.004-2.796-1.356-3.601c-.378-.711-1.46-2.692-1.984-3.332c-1.164-1.332-1.437-1.438-1.809-3.23c-.628-3.067-2.148-8.462-4.042-12.227c-1.004-2-2.626-3.606-4.067-5.07c-1.246-1.247-4.121-3.31-4.668-3.227c-4.766.894-6.226 5.586-8.457 9.27c-1.27 2.062-2.516 3.769-3.52 5.937c-.898 1.98-.812 4.23-2.331 5.938a15.4 15.4 0 0 0-3.333 5.855c-.195.453-.546 4.957-1.003 6.016l7.02-.438c6.585.461 4.687 2.961 14.858 2.438l16.098-.54a25 25 0 0 0-1.433-3.792zM111.793 8.254H16.207C7.312 8.23.086 15.457.086 24.352v35.105c2.352-.812 5.578-5.75 6.668-6.934c1.789-2.062 2.16-4.77 3.059-6.378c2.062-3.793 2.433-6.477 7.101-6.477c2.164 0 3.063.516 4.5 2.516c.996 1.332 2.79 3.957 3.602 5.668c1.004 1.98 2.523 4.582 3.254 5.125c.515.351.972.722 1.433.894c.707.27 1.356-.27 1.902-.629c.622-.539.895-1.52 1.52-2.953c.895-2.086 1.813-4.418 2.332-5.312c.914-1.461 1.273-3.254 2.25-4.067c1.461-1.246 3.441-1.355 3.957-1.437c2.98-.625 4.336 1.437 5.777 2.707c.973.894 2.243 2.605 3.246 4.851c.708 1.793 1.606 3.52 2.067 4.5c.351.98 1.266 2.606 1.789 4.582c.543 1.711 1.809 3.067 2.352 3.961c0 0 .812 2.164 5.476 4.145a35 35 0 0 0 4.336 1.52c2.066.734 4.047.644 6.563.374c1.789 0 2.793-2.625 3.601-4.683c.438-1.254.98-4.774 1.25-5.758c.27-.996-.437-1.707.192-2.625c.722-.977 1.164-1.082 1.519-2.332c.914-2.793 5.957-2.875 8.832-2.875c2.414 0 2.063 2.332 6.125 1.52c2.336-.434 4.586.273 7.023.995c2.063.543 4.043 1.168 5.204 2.524c.73.898 2.629 5.312.73 5.476c.164.188.36.645.625.817c-.46 1.707-2.25.46-3.332.27c-1.355-.27-2.332 0-3.684.624c-2.335.996-5.668.918-7.726 2.625c-1.715 1.438-1.715 4.582-2.543 6.371c0 0-2.254 5.696-6.996 9.192c-1.278.914-3.715 3.058-8.918 3.871c-2.356.355-4.586.355-7.024.27c-1.164-.079-2.332-.079-3.52-.079c-.706 0-3.062-.109-2.96.164l-.27.645c.024.29.063.602.164.895c.102.515.102.976.192 1.437c0 .98-.086 2.063 0 3.066c.082 2.063.894 3.957 1.004 6.102c.078 2.355 1.246 4.875 2.414 6.77c.46.707 1.086.789 1.355 1.71c.352.98 0 2.141.188 3.227c.625 4.227 1.875 8.73 3.773 12.61v.078c2.332-.352 4.77-1.247 7.836-1.684c5.664-.832 13.5-.461 18.54-.914c12.796-1.168 19.706 5.226 31.148 2.601V24.336c-.063-8.895-7.293-16.102-16.207-16.102zM64.086 83.855q0-.28 0 0m-34.457 14.75c.894-1.98 1.433-4.125 2.144-6.101c.73-1.899 1.813-4.61 3.684-5.582c-.246-.274-3.957-.375-4.934-.461c-1.082-.086-2.171-.273-3.25-.438a135 135 0 0 1-6.125-1.265c-1.168-.274-5.21-1.715-6.02-2.067c-2.085-.894-3.421-3.52-4.96-3.246c-.977.188-1.98.54-2.605 1.54c-.543.812-.731 2.242-1.083 3.226c-.437 1.086-1.168 2.164-1.707 3.25c-1.277 1.875-3.332 3.582-4.23 5.484c-.191.457-.27.895-.457 1.356v21.683c1.082.188 2.16.371 3.328.73c8.996 2.438 11.164 2.606 19.98 1.63l.813-.11c.625-1.437 1.188-6.207 1.629-7.644c.352-1.164.812-2.063.996-3.14c.164-1.09 0-2.173-.102-3.15c-.171-2.628 1.895-3.519 2.899-5.69zm0 0\"/></svg>",
    "devicon-plain:opengl": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" fill-rule=\"evenodd\" d=\"M26.289 75.031c-6.137 0-9.203-3.645-10.387-7.453c1.336 6.191 8.555 12.23 31.27 14.91c21.473 2.543 35.465-3.848 40.199-6.828c0 0 3.191-1.363 1.277.746c0 0-13.508 12.129-40.094 12.285C22.02 88.816.926 75.828 1.086 62.648C.957 49.523 22.02 36.504 48.555 36.609c26.531.125 40.094 12.281 40.094 12.281c1.914 2.105-1.277.75-1.277.75c-4.719-2.969-18.676-8.82-40.199-6.828c-21.207 1.984-28.094 8.98-30.41 13.652a13.4 13.4 0 0 0-1.383 5.242c.57-4.496 3.449-9.77 10.871-9.77c8.398 0 10.98 6.828 10.98 11.551s-2.574 11.5-10.98 11.5zm85.57-4.543h11.195v3.965h-15.93V52.555h4.707v17.91zm-17.277-7.824H104v11.813h-3.137l-.469-2.738c-1.195 1.34-2.926 3.324-7.051 3.324c-5.437 0-10.387-3.777-10.387-11.445c0-5.984 3.422-11.605 11.031-11.605c6.891 0 9.633 4.359 9.902 7.352h-4.707c0-.852-1.605-3.598-4.93-3.598c-3.359 0-6.461 2.258-6.461 7.875c0 5.988 3.359 7.512 6.566 7.512c1.035 0 4.488-.395 5.438-4.816h-5.242v-3.687zm-68.348-8.031c-4.898 0-7.855 3.746-7.855 8.926c0 5.148 2.949 8.926 7.855 8.926c4.898 0 7.859-3.75 7.859-8.926c0-5.152-2.949-8.926-7.859-8.926m12.703 5.828h2.285v1.98h.055c.582-.805 1.68-2.387 4.301-2.387c3.832 0 6.566 3.152 6.566 7.09c0 3.344-1.988 7.719-6.891 7.719c-1.93 0-3.195-.883-3.832-1.875h-.055v7.035H38.94V60.488zm6.133 12.395c2.648 0 4.441-2.23 4.441-5.305c0-1.801-.75-5.359-4.496-5.359c-3.496 0-3.883 3.668-3.883 5.938c0 3.715 2.398 4.711 3.938 4.711zm21.742-2.762c-.082.672-.742 2.652-2.562 3.828c-.66.43-1.598.969-3.91.969c-4.055 0-6.457-2.973-6.457-7.039c0-4.336 2.152-7.77 6.941-7.77c4.16 0 6.188 3.215 6.188 8.191H56.465c0 2.922 1.402 4.602 4.191 4.602c2.289 0 3.637-1.707 3.719-2.754h2.43zm-2.57-3.715c-.141-2.172-1.078-4.176-4.027-4.176c-2.234 0-4 2.004-4 4.176zm16.145 8.082h-2.426v-8.609c0-2.434-.719-3.664-3.09-3.664c-1.379 0-3.805.855-3.805 4.656v7.613h-2.43V60.441h2.289v1.977h.055c.523-.75 1.871-2.383 4.355-2.383c2.23 0 5.043.883 5.043 4.875v9.504m47.376-.695a1.95 1.95 0 0 1-.277 1.012a1.97 1.97 0 0 1-.758.742a2.07 2.07 0 0 1-1.035.273c-.367 0-.711-.094-1.031-.273s-.57-.426-.758-.742a1.95 1.95 0 0 1-.277-1.012q.001-.54.277-1.016a2 2 0 0 1 .758-.738a2.07 2.07 0 0 1 1.031-.27c.367 0 .719.09 1.035.27s.574.426.758.738a2 2 0 0 1 .277 1.016m-.398 0a1.53 1.53 0 0 0-.492-1.152c-.324-.324-.719-.484-1.176-.484s-.855.16-1.18.484a1.54 1.54 0 0 0-.488 1.152a1.55 1.55 0 0 0 .488 1.156a1.62 1.62 0 0 0 1.18.477c.461 0 .852-.156 1.176-.477a1.57 1.57 0 0 0 .492-1.156m-2.605-1.086h.988c.277 0 .484.059.609.168s.188.262.188.445q.001.216-.145.379q-.142.154-.453.23a.6.6 0 0 1 .188.098a1 1 0 0 1 .176.223a48 48 0 0 1 .359.625h-.652l-.426-.738c-.07-.082-.145-.125-.215-.125c-.02 0-.039.004-.07.008v.855h-.551zm.551.922h.238q.236 0 .336-.078a.24.24 0 0 0 .098-.195a.23.23 0 0 0-.094-.191q-.093-.077-.32-.078h-.258zm0 0\"/></svg>",
    "devicon-plain:perl": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M53.343 127.515c-13.912-2.458-25.845-8.812-35.707-19.004C9.252 99.845 3.48 88.926.851 76.764C-.284 71.51-.284 56.477.85 51.222c1.776-8.219 5.228-16.388 9.927-23.509c3.112-4.71 12.227-13.825 16.938-16.937c7.121-4.698 15.292-8.15 23.511-9.925c5.256-1.135 20.29-1.135 25.546 0c12.809 2.769 23.454 8.553 32.638 17.736c9.188 9.187 14.969 19.827 17.738 32.635c1.135 5.255 1.135 20.287 0 25.542c-2.769 12.808-8.55 23.452-17.738 32.635c-9.043 9.042-19.55 14.81-32.146 17.652c-4.469 1.005-19.24 1.295-23.922.464zm11.565-12.772c0-4.194-.06-4.496-.908-4.496c-.84 0-.904.29-.868 3.899c.04 4.262.34 5.574 1.207 5.284c.404-.134.57-1.494.57-4.687zm-6.758 1.445c1.196-1.194 1.543-1.917 1.543-3.209c0-1.315-.162-1.634-.763-1.517c-.416.08-.92.759-1.114 1.505c-.198.751-1.002 1.906-1.785 2.572c-1.417 1.194-1.47 2.191-.121 2.191c.384 0 1.393-.694 2.24-1.542m14.945 1.05c.166-.271-.339-1.037-1.126-1.699c-.783-.666-1.587-1.821-1.784-2.572c-.194-.746-.699-1.425-1.115-1.505c-.601-.117-.763.202-.763 1.517c0 2.608 3.747 5.942 4.788 4.259m-20.66-8.146c0-.262-.635-.823-1.41-1.247c-5.058-2.769-10.984-7.177-14.282-10.612c-6.435-6.704-9.33-13.385-9.402-21.676c-.044-5.542.67-8.432 3.367-13.607c2.608-5 5.631-8.779 13.947-17.42c9.29-9.648 11.429-12.195 13.043-15.53c1.147-2.369 1.296-3.232 1.458-8.238c.197-6.216-.182-10.506-.929-10.506c-.339 0-.403 1.614-.21 5.235c.622 11.593-1.53 15.19-14.892 24.88c-9.2 6.677-13.422 10.302-16.612 14.261c-4.517 5.615-6.52 10.471-7.02 17.054c-1.207 15.868 8.85 29.628 26.591 36.385c3.916 1.49 6.35 1.881 6.35 1.021zm30.696-1.287c6.1-2.539 10.738-5.611 15.11-10.007c6.665-6.7 9.442-12.965 9.858-22.24c.363-8.134-1.405-13.515-6.439-19.61c-3.447-4.173-7.161-7.16-17.173-13.812c-13.47-8.95-16.632-12.513-16.632-18.746c0-1.659.299-4.004.662-5.219c.622-2.066.606-3.491-.02-1.857c-.593 1.546-1.946.836-2.676-1.408l-.703-2.156l.267 2.043c.94 7.241 1.061 10.272.641 16.614c-.56 8.565-1.614 14.426-4.505 25.074c-2.87 10.572-3.387 14.402-3.031 22.475c.298 6.826 1.255 11.932 3.475 18.592c2.06 6.188 2.443 6.656 6.23 7.625c2.086.533 4.06 1.433 5.63 2.567c1.474 1.066 2.952 1.76 3.78 1.776c.75.012 3.237-.759 5.526-1.711m-1.369-3.076c-.565-.565-.302-1.046 1.91-3.492c6.972-7.697 10.096-15.645 10.185-25.906c.06-6.995-1.482-11.625-6.197-18.592c-2.135-3.152-9.636-11.011-13.265-13.893c-2.664-2.115-5.397-5.72-5.886-7.762c-.496-2.067.888-1.522 2.495.985c.787 1.227 2.495 3.027 3.79 4c1.297.977 5.132 3.834 8.523 6.357c11.666 8.67 16.858 16.065 18.024 25.668c.679 5.558-.395 11.302-3.108 16.634c-2.81 5.526-7.937 11.545-12.325 14.479c-2.7 1.8-3.552 2.115-4.146 1.522m-22.836.585c.133-.343-1.034-2.535-2.592-4.872c-4.13-6.192-5.926-9.61-7.602-14.454c-1.413-4.09-1.49-4.646-1.501-10.887c-.016-9.433 1.005-12.424 8.49-24.848c7.056-11.722 8.013-16.259 7.217-34.286c-.286-6.462-.61-11.839-.718-11.948c-.747-.746-.904 1.167-.63 7.665c.549 12.941-.287 20.15-3.016 26.064c-1.857 4.024-3.936 7.076-9.53 14.002c-7.788 9.64-9.984 14.75-9.944 23.125c.029 5.744.808 9.276 3.129 14.188c2.51 5.316 7.133 10.685 12.926 15.012c2.669 1.99 3.391 2.228 3.77 1.239z\"/></svg>",
    "devicon-plain:php": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M64 30.332C28.654 30.332 0 45.407 0 64s28.654 33.668 64 33.668c35.345 0 64-15.075 64-33.668S99.346 30.332 64 30.332m-5.982 9.81h7.293v.003l-1.745 8.968h6.496q6.132 0 8.458 2.139q2.328 2.14 1.398 6.93l-3.053 15.7h-7.408l2.902-14.929q.495-2.546-.365-3.473q-.86-.925-3.658-.925h-5.828L58.752 73.88h-7.291zM26.73 49.114h14.133q6.379 0 9.305 3.348q2.925 3.347 1.758 9.346q-.481 2.472-1.625 4.52t-2.99 3.745q-2.202 2.06-4.891 2.936q-2.691.876-6.858.875h-6.294l-1.745 8.97h-7.35zm57.366 0h14.13q6.378 0 9.303 3.348h.002q2.926 3.347 1.76 9.346q-.48 2.472-1.623 4.52t-2.992 3.745q-2.2 2.06-4.893 2.936q-2.69.876-6.855.875h-6.295l-1.744 8.97h-7.35zm-51.051 5.325l-2.742 14.12h4.468q4.446.001 6.622-1.673q2.174-1.675 2.937-5.592q.728-3.762-.666-5.309t-5.584-1.547zm57.363 0l-2.744 14.12h4.47q4.446.001 6.622-1.673q2.173-1.675 2.935-5.592q.73-3.762-.664-5.309t-5.584-1.547z\"/></svg>",
    "devicon-plain:powershell": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" fill-rule=\"evenodd\" d=\"M124.912 19.358c-.962-1.199-2.422-1.858-4.111-1.858h-92.61c-3.397 0-6.665 2.642-7.444 6.015L2.162 104.022c-.396 1.711-.058 3.394.926 4.619c.963 1.199 2.423 1.858 4.111 1.858v.001H99.81c3.396 0 6.665-2.643 7.443-6.016l18.586-80.508c.395-1.711.057-3.395-.927-4.618m-98.589 77.17c-1.743-2.397-1.323-5.673.94-7.318l37.379-27.067v-.556L41.157 36.603c-1.916-2.038-1.716-5.333.445-7.361s5.466-2.019 7.382.019l28.18 29.979c1.6 1.702 1.718 4.279.457 6.264c-.384.774-1.182 1.628-2.593 2.618l-41.45 29.769c-2.263 1.644-5.512 1.034-7.255-1.363m59.543.538H63.532c-2.597 0-4.702-2.082-4.702-4.65s2.105-4.65 4.702-4.65h22.333c2.597 0 4.702 2.082 4.702 4.65s-2.104 4.65-4.701 4.65\" clip-rule=\"evenodd\"/></svg>",
    "devicon-plain:python": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M49.33 62h29.159C86.606 62 93 55.132 93 46.981V19.183c0-7.912-6.632-13.856-14.555-15.176c-5.014-.835-10.195-1.215-15.187-1.191c-4.99.023-9.612.448-13.805 1.191C37.098 6.188 35 10.758 35 19.183V30h29v4H23.776c-8.484 0-15.914 5.108-18.237 14.811c-2.681 11.12-2.8 17.919 0 29.53C7.614 86.983 12.569 93 21.054 93H31V79.952C31 70.315 39.428 62 49.33 62m-1.838-39.11c-3.026 0-5.478-2.479-5.478-5.545c0-3.079 2.451-5.581 5.478-5.581c3.015 0 5.479 2.502 5.479 5.581c-.001 3.066-2.465 5.545-5.479 5.545m74.789 25.921C120.183 40.363 116.178 34 107.682 34H97v12.981C97 57.031 88.206 65 78.489 65H49.33C41.342 65 35 72.326 35 80.326v27.8c0 7.91 6.745 12.564 14.462 14.834c9.242 2.717 17.994 3.208 29.051 0C85.862 120.831 93 116.549 93 108.126V97H64v-4h43.682c8.484 0 11.647-5.776 14.599-14.66c3.047-9.145 2.916-17.799 0-29.529m-41.955 55.606c3.027 0 5.479 2.479 5.479 5.547c0 3.076-2.451 5.579-5.479 5.579c-3.015 0-5.478-2.502-5.478-5.579c0-3.068 2.463-5.547 5.478-5.547\"/></svg>",
    "devicon-plain:r": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M64 14.647c-35.346 0-64 19.19-64 42.864c0 20.764 22.046 38.076 51.316 42.017V86.7C35.766 81.81 25.15 72.005 25.15 60.707c0-16.183 21.78-29.303 48.647-29.303s46.691 8.975 46.691 29.303c0 10.486-5.271 17.95-14.064 22.72c1.204.909 2.218 2.074 2.902 3.42l.389.655C121.025 79.772 128 69.189 128 57.51c0-23.672-28.654-42.863-64-42.863M52.736 41.264v72.084l21.834-.01l-.004-28.219h5.862c1.199 0 1.716.348 2.93 1.33c1.453 1.177 3.816 5.239 3.816 5.239l11.537 21.665l24.674-.01l-15.266-25.74a8.4 8.4 0 0 0-1.42-2.04c-.974-1.037-2.325-1.823-3.105-2.22c-2.25-1.137-6.12-2.307-6.123-2.308c0 0 19.08-1.415 19.08-20.414S96.58 41.264 96.58 41.264zM74.754 56.89l13.219.007s6.123-.33 6.123 6.01c0 6.216-6.123 6.235-6.123 6.235l-13.225.004zM84.1 89.527a102 102 0 0 1-8.114.463l.002 9.63a88 88 0 0 0 12.475-2.491l-.502-.942c-.68-1.268-1.347-2.542-2.033-3.806a41 41 0 0 0-1.828-2.852z\"/></svg>",
    "devicon-plain:ruby": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"m35.971 111.33l81.958 11.188c-9.374-15.606-18.507-30.813-27.713-46.144Zm89.71-86.383L93.513 73.339c-.462.696-1.061 1.248-.41 2.321c8.016 13.237 15.969 26.513 23.942 39.777c1.258 2.095 2.53 4.182 4.157 6.192l4.834-96.58zM16.252 66.22c.375.355 1.311.562 1.747.347c7.689-3.779 15.427-7.474 22.948-11.564c2.453-1.333 4.339-3.723 6.452-5.661c6.997-6.417 13.983-12.847 20.966-19.278c.427-.395.933-.777 1.188-1.275c2.508-4.902 4.973-9.829 7.525-14.898c-3.043-1.144-5.928-2.263-8.849-3.281c-.396-.138-1.02.136-1.449.375c-6.761 3.777-13.649 7.353-20.195 11.472c-3.275 2.061-5.943 5.098-8.843 7.743c-4.674 4.266-9.342 8.542-13.948 12.882a24 24 0 0 0-3.288 3.854c-3.15 4.587-6.206 9.24-9.402 14.025c1.786 1.847 3.41 3.613 5.148 5.259m28.102-6.271l-11.556 48.823l54.3-34.987zm76.631-34.846l-46.15 7.71l15.662 38.096zM44.996 56.644l41.892 13.6c-5.25-12.79-10.32-25.133-15.495-37.737ZM16.831 75.643L2.169 110.691l27.925-.825Zm13.593 26.096l.346-.076c3.353-13.941 6.754-27.786 10.177-42.272L18.544 71.035c3.819 9.926 7.891 20.397 11.88 30.704m84.927-78.897c-4.459-1.181-8.918-2.366-13.379-3.539c-6.412-1.686-12.829-3.351-19.237-5.052c-.801-.213-1.38-.352-1.851.613c-2.265 4.64-4.6 9.245-6.901 13.868c-.071.143-.056.328-.111.687l41.47-6.285zM89.482 12.288l36.343 10.054l-6.005-17.11l-30.285 6.715ZM33.505 114.007c-4.501-.519-9.122-.042-13.687.037c-3.75.063-7.5.206-11.25.323c-.386.012-.771.09-1.156.506q46.504 4.298 93.007 8.6l.063-.414l-29.815-4.07c-12.384-1.691-24.747-3.551-37.162-4.982M2.782 99.994c3.995-9.27 7.973-18.546 11.984-27.809c.401-.929.37-1.56-.415-2.308c-1.678-1.597-3.237-3.318-5.071-5.226c-2.479 12.24-4.897 24.177-7.317 36.113l.271.127c.185-.297.411-.578.548-.897m78.74-90.153c6.737-1.738 13.572-3.097 20.367-4.613c.44-.099.87-.244 1.303-.368l-.067-.332l-29.194 3.928c2.741 1.197 4.853 2.091 7.591 1.385\"/></svg>",
    "devicon-plain:rust": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M62.96.242c-.232.135-1.203 1.528-2.16 3.097c-2.4 3.94-2.426 3.942-5.65.549c-2.098-2.207-2.605-2.611-3.28-2.606c-.44.002-.995.152-1.235.332c-.239.18-.916 1.612-1.504 3.183c-1.346 3.6-1.41 3.715-2.156 3.859c-.46.087-1.343-.406-3.463-1.928c-1.565-1.125-3.1-2.045-3.411-2.045c-1.291 0-1.655.706-2.27 4.4c-.78 4.697-.754 4.681-4.988 2.758c-1.71-.776-3.33-1.411-3.603-1.411c-.274 0-.792.294-1.15.653c-.652.652-.653.655-.475 4.246l.178 3.595l-.68.364c-.602.322-1.017.283-3.684-.348c-3.48-.822-4.216-.8-4.92.15l-.516.693l.692 2.964c.38 1.63.745 3.2.814 3.487c.067.287-.05.746-.26 1.02c-.348.448-.717.489-3.939.44c-5.453-.086-5.762.382-3.51 5.3c.717 1.56 1.304 2.979 1.304 3.149c0 .899-.717 1.225-3.794 1.728c-1.722.28-3.218.51-3.326.51c-.107 0-.43.235-.717.522c-.937.936-.671 1.816 1.453 4.814c2.646 3.735 2.642 3.749-1.73 5.421c-4.971 1.902-5.072 2.37-1.287 5.96c3.525 3.344 3.53 3.295-.461 5.804C.208 62.8.162 62.846.085 63.876c-.093 1.253-.071 1.275 3.538 3.48c3.57 2.18 3.57 2.246.067 5.56C-.078 76.48.038 77 5.013 78.877c4.347 1.64 4.353 1.66 1.702 5.394c-1.502 2.117-1.981 2.999-1.981 3.653c0 1.223.637 1.535 4.441 2.174c3.205.54 3.919.857 3.919 1.741c0 .182-.588 1.612-1.307 3.177c-2.236 4.87-1.981 5.275 3.311 5.275c4.929 0 4.798-.15 3.736 4.294c-.8 3.35-.813 3.992-.088 4.715c.554.556 1.6.494 4.87-.289c2.499-.596 2.937-.637 3.516-.328l.661.354l-.178 3.594c-.178 3.593-.177 3.595.475 4.248c.358.359.884.652 1.165.652c.282 0 1.903-.631 3.604-1.404c4.22-1.916 4.194-1.932 4.973 2.75c.617 3.711.977 4.4 2.294 4.4c.327 0 1.83-.88 3.34-1.958c2.654-1.893 3.342-2.19 4.049-1.74c.182.115.89 1.67 1.572 3.455c1.003 2.625 1.37 3.309 1.929 3.576c1.062.509 1.72.1 4.218-2.62c3.016-3.286 3.14-3.27 5.602.72c2.72 4.406 3.424 4.396 6.212-.089c2.402-3.864 2.374-3.862 5.621-.47c2.157 2.25 2.616 2.61 3.343 2.61c.464 0 1.019-.175 1.23-.388c.214-.213.92-1.786 1.568-3.496c.649-1.71 1.321-3.2 1.495-3.31c.687-.436 1.398-.13 4.048 1.752c1.56 1.108 3.028 1.959 3.377 1.959c1.296 0 1.764-.92 2.302-4.534c.46-3.082.554-3.378 1.16-3.685c.596-.302.954-.2 3.75 1.07c1.701.77 3.323 1.402 3.604 1.402c.282 0 .816-.302 1.184-.672l.672-.67l-.184-3.448c-.177-3.291-.16-3.468.364-3.943c.54-.488.596-.486 3.615.204c3.656.835 4.338.857 5.025.17c.671-.671.664-.818-.254-4.691c-1.03-4.345-1.168-4.19 3.78-4.19c3.374 0 3.75-.048 4.18-.522c.718-.793.547-1.702-.896-4.779c-.729-1.55-1.32-2.96-1.315-3.135c.024-.914.743-1.227 4.065-1.767c2.033-.329 3.553-.711 3.829-.96c.923-.833.584-1.918-1.523-4.873c-2.642-3.703-2.63-3.738 1.599-5.297c5.064-1.866 5.209-2.488 1.419-6.09c-3.51-3.335-3.512-3.317.333-5.677c4.648-2.853 4.655-3.496.082-6.335c-3.933-2.44-3.93-2.406-.405-5.753c3.78-3.593 3.678-4.063-1.295-5.965c-4.388-1.679-4.402-1.72-1.735-5.38c1.588-2.18 1.982-2.903 1.982-3.65c0-1.306-.586-1.598-4.436-2.22c-3.216-.52-3.924-.835-3.924-1.75c0-.174.588-1.574 1.307-3.113c1.406-3.013 1.604-4.22.808-4.94c-.428-.387-1-.443-4.067-.392c-3.208.054-3.618.008-4.063-.439c-.486-.488-.48-.557.278-3.725c.931-3.881.935-3.975.17-4.694c-.777-.73-1.262-.718-4.826.121c-2.597.612-3.027.653-3.617.337l-.67-.36l.185-3.582l.186-3.581l-.67-.67c-.369-.369-.891-.67-1.163-.67c-.27 0-1.884.64-3.583 1.422c-2.838 1.306-3.143 1.393-3.757 1.072c-.612-.32-.714-.637-1.237-3.829c-.603-3.693-.977-4.412-2.288-4.412c-.311 0-1.853.925-3.426 2.055c-2.584 1.856-2.93 2.032-3.574 1.807c-.533-.186-.843-.59-1.221-1.599c-.28-.742-.817-2.172-1.194-3.177c-.762-2.028-1.187-2.482-2.328-2.482c-.637 0-1.213.458-3.28 2.604c-3.249 3.375-3.261 3.374-5.65-.545C66.073 1.78 65.075.382 64.81.24c-.597-.321-1.3-.32-1.85.002m2.96 11.798c2.83 2.014 1.326 6.75-2.144 6.75c-3.368 0-5.064-4.057-2.659-6.36c1.357-1.3 3.303-1.459 4.804-.39m-3.558 12.507c1.855.705 2.616.282 6.852-3.8l3.182-3.07l1.347.18c4.225.56 12.627 4.25 17.455 7.666c4.436 3.14 10.332 9.534 12.845 13.93l.537.942l-2.38 5.364c-1.31 2.95-2.382 5.673-2.382 6.053c0 .878.576 2.267 1.13 2.726c.234.195 2.457 1.265 4.939 2.378l4.51 2.025l.178 1.148c.23 1.495.26 5.167.052 6.21l-.163.816h-2.575c-2.987 0-2.756-.267-2.918 3.396c-.118 2.656-.76 4.124-2.219 5.075c-2.378 1.551-6.305 1.27-7.97-.571c-.256-.283-.753-1.704-1.106-3.16c-1.03-4.253-2.413-6.64-5.193-8.964c-.878-.733-1.595-1.418-1.595-1.522c0-.102.965-.915 2.145-1.803c4.298-3.24 6.77-7.012 7.04-10.747c.519-7.126-5.158-13.767-13.602-15.92c-2.002-.51-2.857-.526-27.624-.526c-14.057 0-25.559-.092-25.559-.204c0-.263 3.124-3.295 4.964-4.816c5.054-4.178 11.618-7.465 18.417-9.221l2.35-.609l3.341 3.387c1.838 1.863 3.64 3.499 4.002 3.637M20.3 46.339c1.539 1.008 2.17 3.54 1.26 5.062c-1.405 2.356-4.966 2.455-6.373.178c-2.046-3.309 1.895-7.349 5.113-5.24m90.672.129c4.026 2.455.906 8.494-3.404 6.587c-2.877-1.273-2.97-5.206-.155-6.641c1.174-.6 2.523-.578 3.56.054m-78.81 15.031v15.02h-13.28l-.526-2.285c-1.036-4.5-1.472-9.156-1.211-12.969l.182-2.679l4.565-2.047c2.864-1.283 4.706-2.262 4.943-2.625c1.038-1.584.94-2.715-.518-5.933l-.68-1.502h6.523v15.02M70.39 47.132c2.843.74 4.345 2.245 4.349 4.355c.002 1.549-.765 2.52-2.67 3.38c-1.348.61-1.562.625-10.063.708l-8.686.084v-8.92h7.782c6.078 0 8.112.086 9.288.393m-2.934 21.554c1.41.392 3.076 1.616 3.93 2.888c.898 1.337 1.423 3.076 2.667 8.836c1.05 4.869 1.727 6.46 3.62 8.532c2.345 2.566 1.8 2.466 13.514 2.466c5.61 0 10.198.09 10.198.2c0 .197-3.863 4.764-4.03 4.764c-.048 0-2.066-.422-4.484-.939c-6.829-1.458-7.075-1.287-8.642 6.032l-1.008 4.702l-.91.448c-1.518.75-6.453 2.292-9.01 2.819c-4.228.87-8.828 1.163-12.871.822c-6.893-.585-16.02-3.259-16.377-4.8c-.075-.327-.535-2.443-1.018-4.704c-.485-2.26-1.074-4.404-1.31-4.764c-1.13-1.724-2.318-1.83-7.547-.674c-1.98.439-3.708.796-3.84.796c-.248 0-3.923-4.249-3.923-4.535c0-.09 8.728-.194 19.396-.23l19.395-.066l.07-6.89c.05-4.865-.018-6.997-.229-7.25c-.235-.284-1.486-.358-6.012-.358H53.32v-8.36l6.597.001c3.626.002 7.02.12 7.539.264M37.57 100.019c3.084 1.88 1.605 6.804-2.043 6.8c-3.74-.001-5.127-4.881-1.94-6.826c1.055-.643 2.908-.63 3.983.026m56.48.206c1.512 1.108 2.015 3.413 1.079 4.949c-2.46 4.035-8.612.828-6.557-3.418c1.01-2.085 3.695-2.837 5.478-1.531\"/></svg>",
    "devicon-plain:scala": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M25.411 110.572V95.077l11.842-.474c12.315-.473 21.45-1.488 34.847-3.789c15.225-2.639 30.246-7.375 31.803-10.082c.406-.677.676 4.534.676 14.616v15.698l-1.76 1.353c-1.894 1.489-9.202 3.993-17.524 6.09C72.303 121.737 40.568 126 29.742 126h-4.33zm0-39.245V55.83l11.842-.406c13.127-.541 23.344-1.691 36.877-4.195c15.157-2.842 28.96-7.443 29.976-9.947c.203-.473.406 6.09.406 14.616c.067 13.533-.068 15.698-1.083 16.78c-2.368 2.64-20.638 7.376-39.449 10.286c-11.435 1.76-30.381 3.79-35.66 3.79h-2.909zm0-38.975V17.195l2.098-.406c1.15-.203 3.992-.406 6.293-.406c11.367 0 38.366-3.722 51.628-7.105c9.27-2.436 15.698-4.872 17.931-6.902c1.15-1.015 1.218-.406 1.218 14.48c0 14.548-.067 15.63-1.285 16.714c-1.827 1.691-14.345 5.548-24.09 7.51c-15.765 3.113-41.951 6.429-50.883 6.429h-2.91z\"/></svg>",
    "devicon-plain:svelte": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M110.293 16.914C98.586-.086 75.668-5 58.02 5.707l-29.856 18.98a33.94 33.94 0 0 0-15.418 22.938a35.54 35.54 0 0 0 3.566 23.102a33 33 0 0 0-5.066 12.793a36.52 36.52 0 0 0 6.191 27.52c11.727 16.96 34.583 21.897 52.27 11.312l29.879-19a34.03 34.03 0 0 0 15.355-22.938a35.44 35.44 0 0 0-3.586-23.02c7.938-12.456 7.52-28.48-1.062-40.48m-55.254 95.773a23.645 23.645 0 0 1-25.394-9.433c-3.461-4.793-4.73-10.711-3.73-16.586l.585-2.832l.54-1.75l1.605 1.062c3.52 2.668 7.46 4.582 11.668 5.875l1.082.375l-.122 1.067c-.105 1.48.332 3.144 1.188 4.414c1.75 2.52 4.793 3.73 7.727 2.937c.644-.207 1.273-.418 1.812-.754l29.754-18.976c1.5-.961 2.457-2.398 2.832-4.106c.328-1.773-.106-3.585-1.066-5.02c-1.774-2.46-4.793-3.565-7.727-2.831c-.645.226-1.332.48-1.879.812l-11.25 7.145c-10.644 6.328-24.394 3.355-31.46-6.832a21.85 21.85 0 0 1-3.75-16.586a20.64 20.64 0 0 1 9.456-13.875l29.692-18.98c1.875-1.168 3.894-2.02 6.082-2.668c9.605-2.5 19.726 1.27 25.394 9.394a22.03 22.03 0 0 1 3.043 19.398l-.543 1.77l-1.539-1.062a39.4 39.4 0 0 0-11.727-5.875l-1.066-.313l.106-1.066c.105-1.563-.332-3.207-1.188-4.48c-1.754-2.52-4.793-3.583-7.727-2.833c-.644.211-1.273.418-1.812.754L45.812 49.977c-1.5 1-2.46 2.394-2.773 4.144c-.312 1.707.106 3.582 1.066 4.957c1.708 2.524 4.81 3.586 7.688 2.832c.687-.207 1.332-.414 1.855-.75l11.375-7.254c1.856-1.226 3.938-2.12 6.067-2.707c9.668-2.52 19.75 1.274 25.394 9.438c3.461 4.793 4.793 10.707 3.832 16.52a20.5 20.5 0 0 1-9.332 13.874L61.23 109.97a25.8 25.8 0 0 1-6.187 2.707zm0 0\"/></svg>",
    "devicon-plain:swift": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M125.54 26.23a28.8 28.8 0 0 0-2.65-7.58a28.8 28.8 0 0 0-4.76-6.32a23.4 23.4 0 0 0-6.62-4.55a27.3 27.3 0 0 0-7.68-2.53c-2.65-.51-5.56-.51-8.21-.76H30.25a45.5 45.5 0 0 0-6.09.51a21.8 21.8 0 0 0-5.82 1.52c-.53.25-1.32.51-1.85.76a34 34 0 0 0-5 3.28c-.53.51-1.06.76-1.59 1.26a22.4 22.4 0 0 0-4.76 6.32a23.6 23.6 0 0 0-2.65 7.58a79 79 0 0 0-.79 7.83v60.39a39.3 39.3 0 0 0 .79 7.83a28.8 28.8 0 0 0 2.65 7.58a28.8 28.8 0 0 0 4.76 6.32a23.4 23.4 0 0 0 6.62 4.55a27.3 27.3 0 0 0 7.68 2.53c2.65.51 5.56.51 8.21.76h63.22a45 45 0 0 0 8.21-.76a27.3 27.3 0 0 0 7.68-2.53a30 30 0 0 0 6.62-4.55a22.4 22.4 0 0 0 4.76-6.32a23.6 23.6 0 0 0 2.65-7.58a79 79 0 0 0 .79-7.83V34.06a39.3 39.3 0 0 0-.8-7.83m-18.79 75.54C101 91 90.37 94.33 85 96.5c-11.11 6.13-26.38 6.76-41.75.47A64.53 64.53 0 0 1 13.84 73a50 50 0 0 0 10.85 6.32c15.87 7.1 31.73 6.61 42.9 0c-15.9-11.66-29.4-26.82-39.46-39.2a43.5 43.5 0 0 1-5.29-6.82c12.16 10.61 31.5 24 38.38 27.79a272 272 0 0 1-27-32.34a266.8 266.8 0 0 0 44.47 34.87c.71.38 1.26.7 1.7 1a33 33 0 0 0 1.21-3.51c3.71-12.89-.53-27.54-9.79-39.67C93.25 33.81 106 57.05 100.66 76.51c-.14.53-.29 1-.45 1.55l.19.22c10.6 12.63 7.67 26.02 6.35 23.49\"/></svg>",
    "devicon-plain:terraform": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" fill-rule=\"evenodd\" d=\"M46.324 26.082L77.941 44.5v36.836L46.324 62.918zM81.41 44.5v36.836l31.633-18.418V26.082zM11.242 5.523V42.36L42.86 60.777V23.941zm66.699 79.852L46.324 66.957v36.824L77.941 122.2zm0 0\"/></svg>",
    "devicon-plain:typescript": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M2 63.91v62.5h125v-125H2zm100.73-5a15.56 15.56 0 0 1 7.82 4.5a20.6 20.6 0 0 1 3 4c0 .16-5.4 3.81-8.69 5.85c-.12.08-.6-.44-1.13-1.23a7.09 7.09 0 0 0-5.87-3.53c-3.79-.26-6.23 1.73-6.21 5a4.6 4.6 0 0 0 .54 2.34c.83 1.73 2.38 2.76 7.24 4.86c8.95 3.85 12.78 6.39 15.16 10c2.66 4 3.25 10.46 1.45 15.24c-2 5.2-6.9 8.73-13.83 9.9a38.3 38.3 0 0 1-9.52-.1A23 23 0 0 1 80 109.19c-1.15-1.27-3.39-4.58-3.25-4.82a9 9 0 0 1 1.15-.73l4.6-2.64l3.59-2.08l.75 1.11a16.8 16.8 0 0 0 4.74 4.54c4 2.1 9.46 1.81 12.16-.62a5.43 5.43 0 0 0 .69-6.92c-1-1.39-3-2.56-8.59-5c-6.45-2.78-9.23-4.5-11.77-7.24a16.5 16.5 0 0 1-3.43-6.25a25 25 0 0 1-.22-8c1.33-6.23 6-10.58 12.82-11.87a31.7 31.7 0 0 1 9.49.26zm-29.34 5.24v5.12H57.16v46.23H45.65V69.26H29.38v-5a49 49 0 0 1 .14-5.16c.06-.08 10-.12 22-.1h21.81z\"/></svg>",
    "devicon-plain:vim": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M72.6 80.5c.2.2.6.5.9.5h5.3c.3 0 .7-.3.9-.5l1.4-1.5q.3-.3.3-.6l1.5-5.1c.1-.5 0-1-.3-1.3l-1.1-.9c-.2-.2-.6-.1-.9-.1h-4.8l-.2-.2l-.1-.1c-.2 0-.4-.1-.6.1L73 72c-.2 0-.3.5-.4.7L71 77.6c-.2.5-.1 1.1.3 1.5zm.8 26.4l-.4.1h-1.2L79 85.9c.2-.7-.1-1.5-.8-1.7l-.4-.1H65.7c-.5.1-.9.5-1 1l-.7 2.5c-.2.7.3 1.3 1 1.5l.3-.1h1.8l-7.3 20.9c-.2.7.1 1.6.8 1.9l.4.3h11.2c.6 0 1.1-.5 1.3-1.1l.7-2.4c.3-.7-.1-1.5-.8-1.7m53.1-19.7l-1.9-2.5v-.1c-.3-.3-.6-.6-1-.6h-7.2c-.4 0-.7.4-1 .6l-2 2.4h-3.1l-2.1-2.4v-.1c-.2-.3-.6-.5-1-.5h-4l20.2-20.2l-22.6-22.4L121 20.6v-9L118.2 8H77.3L74 11.5v2.9L62.7 3L55 10.5L52.6 8H12.2L9 11.7v9.4l3 2.9h3v26.1l-14 14l14 14v32l5.2 2.9h11.6l9.1-9.5l21.6 21.6L77 110.6c.1.4.4.5.9.7l.4-.2h9.4c.6 0 1.1-.1 1.2-.6l.7-2c.2-.7-.1-1.3-.8-1.5l-.4.1H88l3.4-10.7l2.3-2.3h5l-5 15.9c-.2.7.2 1.1.9 1.4l.4-.2h9.1c.5 0 1-.1 1.2-.6l.8-1.8c.3-.7-.1-1.3-.7-1.6c-.1-.1-.3 0-.5 0h-.4l4.2-13h6.1l-5.1 15.9c-.2.7.2 1.1.9 1.3l.4-.3h10c.5 0 1-.1 1.2-.6l.8-2c.3-.7-.1-1.3-.8-1.5c-.1-.1-.3.1-.5.1h-.7l5.6-18.5c.2-.5.1-1.1-.1-1.4M62.7 4.9L74 16.2v4.7l3.4 4.1H79L50 53V25h3.3l2.7-4.2v-8.9l-.2-.3zM2.9 64.1L15 52v24.2zm38.9 38.3l58.4-60l21.4 21.5l-20.2 20.2h-.1c-.3.1-.5.3-.7.5L98.5 87h-2.9l-2.2-2.4c-.2-.3-.6-.6-1-.6h-8.8c-.6 0-1.1.4-1.3 1l-.8 2.5c-.2.7.1 1.3.8 1.6h1.5L77.4 108l-15.1 15.2z\"/></svg>",
    "devicon-plain:visualbasic": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M64 0A64 64 0 0 0 0 64a64 64 0 0 0 64 64a64 64 0 0 0 64-64A64 64 0 0 0 64 0m-3.76 38.7l6.34.1L48 89.202h-6.58L23.14 38.8h6.579l14 40a23.7 23.7 0 0 1 1.02 4.46h.141a22 22 0 0 1 1.119-4.56zm13.6.1h14.34A15.68 15.68 0 0 1 98.54 42a10.34 10.34 0 0 1 3.84 8.34a12.26 12.26 0 0 1-2.38 7.44a12.52 12.52 0 0 1-6.4 4.501v.139a12.82 12.82 0 0 1 8.16 3.84a11.84 11.84 0 0 1 3.06 8.461a13.18 13.18 0 0 1-4.64 10.48a17.28 17.28 0 0 1-11.7 4H73.84zm12.7 5.26l-6.7.06V60.4h5.999a11.48 11.48 0 0 0 7.58-2.4a8.14 8.14 0 0 0 2.781-6.6q0-7.34-9.66-7.34m-6.7 21.641v18.14h8a12 12 0 0 0 8-2.46a8.42 8.42 0 0 0 2.86-6.74q0-8.92-12.16-8.92z\"/></svg>",
    "devicon-plain:vuejs": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"m0 8.934l49.854.158l14.3 24.415l14.3-24.415l49.548-.158l-63.835 110.134zm126.987.637l-24.37.021l-38.473 66.053L25.692 9.592l-24.75-.02l63.212 107.89z\"/></svg>",
    "devicon-plain:xml": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"M23.25 37.459c-2.661.114-2.363 3.885-2.875 5.797c-.837 3.125-1.608 7.1-1.853 10.324c-.246 3.138-.044 6.046.353 9.219c.309 2.467 1.02 4.104.373 4.767c-.52.623-17.199 15.92-18.543 16.786c-1.392.903-.428 3.13.37 4.379c.841 1.317 1.24 1.999 2.99 1.765c1.645-.222 2.167-2.182 3.127-3.248c1.146-1.353 12.917-12.824 13.957-12.824l-.026-.147c.421 2.923 1.972 5.705 3.598 8.112c.798 1.19 1.68 2.352 2.699 3.37c.936.937 2.145 1.702 3.145 2.409c2.352.123 4.246-1.618 1.875-3.59c-.905-.757-3.935-5.212-4.961-7.676c-.957-2.303-3.074-5.762-.782-7.654c7.921-6.6 19.887-12.132 22.711-12.406c-1.377 5.422-7.257 16.32-9.408 21.48c-.741 1.785-1.233 3.92-.982 5.864c.297 2.324 2.046 4.152 3.144 4.326c2.229-.127 3.74-2.086 5.06-3.463c1.832-1.908 3.396-3.672 5.221-5.588c1.812-1.907 3.593-3.918 5.2-5.734c.988-1.12 3.6-3.829 5.152-3.829c-.584 3.732-3.832 8.33-4.486 12.094c-.667 3.85 2.844 8.563 6.52 5.131c3.336-3.117 6.72-6.056 10-9.22c1.162-1.112.935.2 1.196 1.296c.263 1.114.945 4.875 1.473 5.89c.968 1.857 2.653 3.176 4.799 3.212c1.951.032 3.897-.862 5.258-2.219c1.015-1.011 2.757-3.749.83-4.463c-1.23-.456-1.159-.943-2.65-.693c-.925.158-1.94.395-2.717.959c-1.967-.008-2.042-4.756-1.737-6.215c.36-1.741 1.107-3.522.992-5.34c-.135-2.24-2.554-3.707-3.115-3.865c-.832-.41-1.343.117-2.447 1.133c-.928.857-1.947 1.778-2.863 2.646c-1.829 1.733-4.11 4.36-5.041 4.36c1.634-3.73 3.8-14.16-3.442-13.24c-3.01.384-5.085 2.02-7.457 3.806a253 253 0 0 1-5.014 4.674c-1.269 1.146-1.99 2.152-2.654 2.152c.707-3.962 9.02-16.976 4.74-20.644c-.858-.722-1.678-1.742-2.761-1.864c-1.511.186-3.002.552-4.457 1.016c-3.023 1.043-14.846 6.9-22.399 12.306c-.701.508-1.15-.451-1.236-1.558c-.472-5.732 1.06-11.749 2.654-17.1c1.103-3.732-2.992-6.563-3.531-6.693m96.842 4.877c-.889 1.952-1.067 4.788-1.332 7.06v.008c-2.02-.238-4.646-.495-6.668-.617c1.273 1.586 3.698 3.149 5.658 3.88c-.261 1.958-2.326 3.584-3.207 5.476c2.448-.679 3.62-2.514 5.291-4.403c1.935 1.178 4.227 3.363 6.742 3.41c-.952-2.022-1.792-3.723-3.525-5.183c1.44-.94 3.842-1.927 4.949-3.387c-1.844-.396-3.832-.5-5.629-.08c-1.38-1.523-1.605-4.141-2.28-6.164zm-14.733.387c-1.233.113-3.28 2.633-4.453 4.144c-4.323 5.564-8.265 16.89-8.9 18.977c-1.935 6.398-2.686 14.285 1.502 19.916c2.436 3.284 6.728 3.532 10.935 2.324c2.17-.624 4.344-2.181 5.086-3.875c.615-1.411.932-4.037-2.166-2.035c-1.198.773-3.395 1.627-7.334 1.564c-1.32-.024-3.49-1.92-4.072-3.88c-1.024-3.194-.588-7.458-.588-7.458c.892-5.505 3.447-11.47 5.688-16.474c.869-1.932 3.517-6.748 4.51-8.283c0 .004 1.27-1.997.7-4.123l-.01.017c-.167-.624-.487-.852-.898-.814m16.45 15.935c-.647 1.417-.774 3.483-.965 5.133h-.01c-1.464-.17-3.38-.358-4.836-.45c.924 1.152 2.686 2.285 4.106 2.817c-.18 1.416-1.682 2.606-2.325 3.975c1.781-.493 2.639-1.829 3.844-3.194c1.404.845 3.074 2.436 4.899 2.471c-.679-1.471-1.294-2.709-2.555-3.764c1.039-.678 2.78-1.408 3.582-2.459c-1.333-.289-2.789-.36-4.082-.056c-1.003-1.107-1.167-3-1.658-4.473m-6.55 12.95c-.443.98-.533 2.4-.665 3.546l-.012-.004a62 62 0 0 0-3.344-.308c.635.8 1.853 1.58 2.836 1.949c-.127.98-1.16 1.794-1.605 2.742c1.23-.345 1.82-1.264 2.658-2.213c.979.595 2.125 1.689 3.387 1.713c-.472-1.015-.894-1.863-1.762-2.598c.717-.472 1.92-.972 2.479-1.7c-.927-.2-1.927-.252-2.823-.042c-.694-.761-.811-2.07-1.148-3.086z\"/></svg>",
    "devicon-plain:yaml": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 128 128\"><path fill=\"currentColor\" d=\"m.5 5.629l21.754 34.15v21.646h13.959V39.779l22.775-34.15h-15.02L30.021 27.617L16.189 5.629zm63.994.13l-23.66 55.798h11.189l5.139-12.408h25.266l4.252 12.408h11.957L75.937 5.76zm5.992 11.774l7.744 20.475H61.843zm16.195 50.139v54.45H127.5v-11.636H98.636V67.672zm-64.428.011v54.687h11.734V84.647l12.28 25.355H55.5l12.7-26.246v38.602h11.256V67.682h-15.37L50.45 92.414L37.464 67.682z\"/></svg>",
    "mdi:account": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 4a4 4 0 0 1 4 4a4 4 0 0 1-4 4a4 4 0 0 1-4-4a4 4 0 0 1 4-4m0 10c4.42 0 8 1.79 8 4v2H4v-2c0-2.21 3.58-4 8-4\"/></svg>",
    "mdi:alert-circle-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M11 15h2v2h-2zm0-8h2v6h-2zm1-5C6.47 2 2 6.5 2 12a10 10 0 0 0 10 10a10 10 0 0 0 10-10A10 10 0 0 0 12 2m0 18a8 8 0 0 1-8-8a8 8 0 0 1 8-8a8 8 0 0 1 8 8a8 8 0 0 1-8 8\"/></svg>",
    "mdi:alpha-a-box-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zm2 0v14h14V5zm6 2h2a2 2 0 0 1 2 2v8h-2v-4h-2v4H9V9a2 2 0 0 1 2-2m0 2v2h2V9z\"/></svg>",
    "mdi:alpha-d-box-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M9 7h4a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2H9zm2 2v6h2V9zM3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zm2 0v14h14V5z\"/></svg>",
    "mdi:alpha-i-box-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M14 7v2h-1v6h1v2h-4v-2h1V9h-1V7zM5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2m0 2v14h14V5z\"/></svg>",
    "mdi:alpha-l-box-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M9 7h2v8h4v2H9zM5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2m0 2v14h14V5z\"/></svg>",
    "mdi:alpha-u-box-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M9 7h2v8h2V7h2v8a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2zM5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2m0 2v14h14V5z\"/></svg>",
    "mdi:apple": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47c-1.34.03-1.77-.79-3.29-.79c-1.53 0-2 .77-3.27.82c-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51c1.28-.02 2.5.87 3.29.87c.78 0 2.26-1.07 3.81-.91c.65.03 2.47.26 3.64 1.98c-.09.06-2.17 1.28-2.15 3.81c.03 3.02 2.65 4.03 2.68 4.04c-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5c.13 1.17-.34 2.35-1.04 3.19c-.69.85-1.83 1.51-2.95 1.42c-.15-1.15.41-2.35 1.05-3.11\"/></svg>",
    "mdi:book-open-page-variant": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m19 2l-5 4.5v11l5-4.5zM6.5 5C4.55 5 2.45 5.4 1 6.5v14.66c0 .25.25.5.5.5c.1 0 .15-.07.25-.07c1.35-.65 3.3-1.09 4.75-1.09c1.95 0 4.05.4 5.5 1.5c1.35-.85 3.8-1.5 5.5-1.5c1.65 0 3.35.31 4.75 1.06c.1.05.15.03.25.03c.25 0 .5-.25.5-.5V6.5c-.6-.45-1.25-.75-2-1V19c-1.1-.35-2.3-.5-3.5-.5c-1.7 0-4.15.65-5.5 1.5V6.5C10.55 5.4 8.45 5 6.5 5\"/></svg>",
    "mdi:calendar": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M19 19H5V8h14m-3-7v2H8V1H6v2H5c-1.11 0-2 .89-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2h-1V1m-1 11h-5v5h5z\"/></svg>",
    "mdi:check-circle": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10s10-4.5 10-10S17.5 2 12 2m-2 15l-5-5l1.41-1.41L10 14.17l7.59-7.59L19 8z\"/></svg>",
    "mdi:chevron-down": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M7.41 8.58L12 13.17l4.59-4.59L18 10l-6 6l-6-6z\"/></svg>",
    "mdi:chip": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M6 4h12v1h3v2h-3v2h3v2h-3v2h3v2h-3v2h3v2h-3v1H6v-1H3v-2h3v-2H3v-2h3v-2H3V9h3V7H3V5h3zm5 11v3h1v-3zm2 0v3h1v-3zm2 0v3h1v-3z\"/></svg>",
    "mdi:code-braces": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M8 3a2 2 0 0 0-2 2v4a2 2 0 0 1-2 2H3v2h1a2 2 0 0 1 2 2v4a2 2 0 0 0 2 2h2v-2H8v-5a2 2 0 0 0-2-2a2 2 0 0 0 2-2V5h2V3m6 0a2 2 0 0 1 2 2v4a2 2 0 0 0 2 2h1v2h-1a2 2 0 0 0-2 2v4a2 2 0 0 1-2 2h-2v-2h2v-5a2 2 0 0 1 2-2a2 2 0 0 1-2-2V5h-2V3z\"/></svg>",
    "mdi:code-json": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M5 3h2v2H5v5a2 2 0 0 1-2 2a2 2 0 0 1 2 2v5h2v2H5c-1.07-.27-2-.9-2-2v-4a2 2 0 0 0-2-2H0v-2h1a2 2 0 0 0 2-2V5a2 2 0 0 1 2-2m14 0a2 2 0 0 1 2 2v4a2 2 0 0 0 2 2h1v2h-1a2 2 0 0 0-2 2v4a2 2 0 0 1-2 2h-2v-2h2v-5a2 2 0 0 1 2-2a2 2 0 0 1-2-2V5h-2V3zm-7 12a1 1 0 0 1 1 1a1 1 0 0 1-1 1a1 1 0 0 1-1-1a1 1 0 0 1 1-1m-4 0a1 1 0 0 1 1 1a1 1 0 0 1-1 1a1 1 0 0 1-1-1a1 1 0 0 1 1-1m8 0a1 1 0 0 1 1 1a1 1 0 0 1-1 1a1 1 0 0 1-1-1a1 1 0 0 1 1-1\"/></svg>",
    "mdi:code-tags": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m14.6 16.6l4.6-4.6l-4.6-4.6L16 6l6 6l-6 6zm-5.2 0L4.8 12l4.6-4.6L8 6l-6 6l6 6z\"/></svg>",
    "mdi:cog-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 8a4 4 0 0 1 4 4a4 4 0 0 1-4 4a4 4 0 0 1-4-4a4 4 0 0 1 4-4m0 2a2 2 0 0 0-2 2a2 2 0 0 0 2 2a2 2 0 0 0 2-2a2 2 0 0 0-2-2m-2 12c-.25 0-.46-.18-.5-.42l-.37-2.65c-.63-.25-1.17-.59-1.69-.99l-2.49 1.01c-.22.08-.49 0-.61-.22l-2-3.46a.493.493 0 0 1 .12-.64l2.11-1.66L4.5 12l.07-1l-2.11-1.63a.493.493 0 0 1-.12-.64l2-3.46c.12-.22.39-.31.61-.22l2.49 1c.52-.39 1.06-.73 1.69-.98l.37-2.65c.04-.24.25-.42.5-.42h4c.25 0 .46.18.5.42l.37 2.65c.63.25 1.17.59 1.69.98l2.49-1c.22-.09.49 0 .61.22l2 3.46c.13.22.07.49-.12.64L19.43 11l.07 1l-.07 1l2.11 1.63c.19.15.25.42.12.64l-2 3.46c-.12.22-.39.31-.61.22l-2.49-1c-.52.39-1.06.73-1.69.98l-.37 2.65c-.04.24-.25.42-.5.42zm1.25-18l-.37 2.61c-1.2.25-2.26.89-3.03 1.78L5.44 7.35l-.75 1.3L6.8 10.2a5.55 5.55 0 0 0 0 3.6l-2.12 1.56l.75 1.3l2.43-1.04c.77.88 1.82 1.52 3.01 1.76l.37 2.62h1.52l.37-2.61c1.19-.25 2.24-.89 3.01-1.77l2.43 1.04l.75-1.3l-2.12-1.55c.4-1.17.4-2.44 0-3.61l2.11-1.55l-.75-1.3l-2.41 1.04a5.42 5.42 0 0 0-3.03-1.77L12.75 4z\"/></svg>",
    "mdi:console": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M20 19V7H4v12zm0-16a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zm-7 14v-2h5v2zm-3.42-4L5.57 9H8.4l3.3 3.3c.39.39.39 1.03 0 1.42L8.42 17H5.59z\"/></svg>",
    "mdi:cube-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M21 16.5c0 .38-.21.71-.53.88l-7.9 4.44c-.16.12-.36.18-.57.18s-.41-.06-.57-.18l-7.9-4.44A.99.99 0 0 1 3 16.5v-9c0-.38.21-.71.53-.88l7.9-4.44c.16-.12.36-.18.57-.18s.41.06.57.18l7.9 4.44c.32.17.53.5.53.88zM12 4.15L6.04 7.5L12 10.85l5.96-3.35zM5 15.91l6 3.38v-6.71L5 9.21zm14 0v-6.7l-6 3.37v6.71z\"/></svg>",
    "mdi:database-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 3C7.58 3 4 4.79 4 7v10c0 2.21 3.59 4 8 4s8-1.79 8-4V7c0-2.21-3.58-4-8-4m6 14c0 .5-2.13 2-6 2s-6-1.5-6-2v-2.23c1.61.78 3.72 1.23 6 1.23s4.39-.45 6-1.23zm0-4.55c-1.3.95-3.58 1.55-6 1.55s-4.7-.6-6-1.55V9.64c1.47.83 3.61 1.36 6 1.36s4.53-.53 6-1.36zM12 9C8.13 9 6 7.5 6 7s2.13-2 6-2s6 1.5 6 2s-2.13 2-6 2\"/></svg>",
    "mdi:file-compare": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M10 18H6v-2h4zm0-4H6v-2h4zm0-13v1H6c-1.11 0-2 .89-2 2v16a2 2 0 0 0 2 2h4v1h2V1zm10 7v12c0 1.11-.89 2-2 2h-4v-2h4v-9h-4V9h4.5L14 4.5V2zm-4 6h-2v-2h2zm0 4h-2v-2h2z\"/></svg>",
    "mdi:file-document-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M6 2a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6zm0 2h7v5h5v11H6zm2 8v2h8v-2zm0 4v2h5v-2z\"/></svg>",
    "mdi:file-tree": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M3 3h6v4H3zm12 7h6v4h-6zm0 7h6v4h-6zm-2-4H7v5h6v2H5V9h2v2h6z\"/></svg>",
    "mdi:file-tree-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 13H7v5h5v2H5V10h2v1h5zM8 4v2H4V4zm2-2H2v6h8zm10 9v2h-4v-2zm2-2h-8v6h8zm-2 9v2h-4v-2zm2-2h-8v6h8z\"/></svg>",
    "mdi:fish": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m12 20l.76-3c-3.26-.21-6.17-1.6-7.01-3.42c-.09.48-.22.92-.42 1.25C4.67 16 3.33 16 2 16c1.1 0 1.5-1.57 1.5-3.5S3.1 9 2 9c1.33 0 2.67 0 3.33 1.17c.2.33.33.77.42 1.25c.65-1.42 2.57-2.57 4.91-3.1L9 5c2 0 4 0 5.33.67c1.13.56 1.78 1.6 2.36 2.71c2.92.7 5.31 2.28 5.31 4.12c0 1.88-2.5 3.5-5.5 4.16c-.83 1.1-1.64 2.12-2.33 2.67c-.84.67-1.5.67-2.17.67m5-9a1 1 0 0 0-1 1a1 1 0 0 0 1 1a1 1 0 0 0 1-1a1 1 0 0 0-1-1\"/></svg>",
    "mdi:forest": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M16 12L9 2L2 12h1.86L0 18h7v4h4v-4h7l-3.86-6zm4.14 0H22L15 2l-2.39 3.41L17.92 13h-1.95l3.22 5H24zM13 19h4v3h-4z\"/></svg>",
    "mdi:git": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M2.6 10.59L8.38 4.8l1.69 1.7c-.24.85.15 1.78.93 2.23v5.54c-.6.34-1 .99-1 1.73a2 2 0 0 0 2 2a2 2 0 0 0 2-2c0-.74-.4-1.39-1-1.73V9.41l2.07 2.09c-.07.15-.07.32-.07.5a2 2 0 0 0 2 2a2 2 0 0 0 2-2a2 2 0 0 0-2-2c-.18 0-.35 0-.5.07L13.93 7.5a1.98 1.98 0 0 0-1.15-2.34c-.43-.16-.88-.2-1.28-.09L9.8 3.38l.79-.78c.78-.79 2.04-.79 2.82 0l7.99 7.99c.79.78.79 2.04 0 2.82l-7.99 7.99c-.78.79-2.04.79-2.82 0L2.6 13.41c-.79-.78-.79-2.04 0-2.82\"/></svg>",
    "mdi:graph": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M19.5 17c-.13 0-.26 0-.39.04l-1.61-3.25a2.5 2.5 0 0 0-1.75-4.29c-.13 0-.25 0-.39.04l-1.63-3.25c.48-.45.77-1.08.77-1.79a2.5 2.5 0 0 0-5 0c0 .71.29 1.34.76 1.79L8.64 9.54c-.14-.04-.26-.04-.39-.04a2.5 2.5 0 0 0-1.75 4.29l-1.61 3.25C4.76 17 4.63 17 4.5 17a2.5 2.5 0 0 0 0 5A2.5 2.5 0 0 0 7 19.5c0-.7-.29-1.34-.76-1.79l1.62-3.25c.14.04.26.04.39.04s.25 0 .39-.04l1.63 3.25c-.47.45-.77 1.09-.77 1.79a2.5 2.5 0 0 0 5 0A2.5 2.5 0 0 0 12 17c-.13 0-.26 0-.39.04L10 13.79c.46-.45.75-1.08.75-1.79s-.29-1.34-.75-1.79l1.61-3.25c.13.04.26.04.39.04s.26 0 .39-.04L14 10.21c-.45.45-.75 1.09-.75 1.79a2.5 2.5 0 0 0 2.5 2.5c.13 0 .25 0 .39-.04l1.63 3.25c-.47.45-.77 1.09-.77 1.79a2.5 2.5 0 0 0 5 0a2.5 2.5 0 0 0-2.5-2.5\"/></svg>",
    "mdi:graph-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M19.5 17c-.14 0-.26 0-.39.04L17.5 13.8c.45-.45.75-1.09.75-1.8a2.5 2.5 0 0 0-2.5-2.5c-.14 0-.25 0-.4.04L13.74 6.3c.47-.46.76-1.09.76-1.8a2.5 2.5 0 0 0-5 0c0 .7.29 1.34.76 1.79L8.65 9.54c-.15-.04-.26-.04-.4-.04a2.5 2.5 0 0 0-2.5 2.5c0 .71.29 1.34.75 1.79l-1.61 3.25C4.76 17 4.64 17 4.5 17a2.5 2.5 0 0 0 0 5A2.5 2.5 0 0 0 7 19.5c0-.7-.29-1.34-.76-1.79l1.62-3.25c.14.04.26.04.39.04s.25 0 .38-.04l1.63 3.25c-.47.45-.76 1.09-.76 1.79a2.5 2.5 0 0 0 5 0A2.5 2.5 0 0 0 12 17c-.13 0-.26 0-.39.04L10 13.8c.45-.45.75-1.09.75-1.8c0-.7-.29-1.33-.75-1.79l1.61-3.25c.13.04.26.04.39.04s.26 0 .39-.04L14 10.21a2.5 2.5 0 0 0 1.75 4.29c.13 0 .25 0 .38-.04l1.63 3.25c-.47.45-.76 1.09-.76 1.79a2.5 2.5 0 0 0 5 0a2.5 2.5 0 0 0-2.5-2.5m-15 3.5c-.55 0-1-.45-1-1s.45-1 1-1s1 .45 1 1s-.45 1-1 1m8.5-1c0 .55-.45 1-1 1s-1-.45-1-1s.45-1 1-1s1 .45 1 1M7.25 12c0-.55.45-1 1-1s1 .45 1 1s-.45 1-1 1s-1-.45-1-1M11 4.5c0-.55.45-1 1-1s1 .45 1 1s-.45 1-1 1s-1-.45-1-1m3.75 7.5c0-.55.45-1 1-1s1 .45 1 1s-.45 1-1 1s-1-.45-1-1m4.75 8.5c-.55 0-1-.45-1-1s.45-1 1-1s1 .45 1 1s-.45 1-1 1\"/></svg>",
    "mdi:hammer-wrench": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m13.78 15.3l6 6l2.11-2.16l-6-6zm3.72-5.2c-.39 0-.81-.05-1.14-.19L4.97 21.25l-2.11-2.11l7.41-7.4L8.5 9.96l-.72.7l-1.45-1.41v2.86l-.7.7l-3.52-3.56l.7-.7h2.81l-1.4-1.41l3.56-3.56a2.976 2.976 0 0 1 4.22 0L9.89 5.74l1.41 1.4l-.71.71l1.79 1.78l1.82-1.88c-.14-.33-.2-.75-.2-1.12a3.49 3.49 0 0 1 3.5-3.52c.59 0 1.11.14 1.58.42L16.41 6.2l1.5 1.5l2.67-2.67c.28.47.42.97.42 1.6c0 1.92-1.55 3.47-3.5 3.47\"/></svg>",
    "mdi:head-question": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M13 3C9.23 3 6.19 5.95 6 9.66l-1.92 2.53c-.24.31 0 .81.42.81H6v3c0 1.11.89 2 2 2h1v3h7v-4.69c2.37-1.12 4-3.51 4-6.31c0-3.86-3.12-7-7-7m1 11h-2v-2h2zm1.75-5.19c-.29.4-.66.69-1.11.93c-.25.16-.42.33-.51.52c-.09.18-.13.43-.13.74h-2c0-.5.11-.92.31-1.18c.19-.27.54-.57 1.05-.91c.26-.16.47-.36.61-.59c.16-.23.23-.5.23-.82c0-.3-.08-.56-.26-.75c-.18-.18-.44-.28-.75-.28a1 1 0 0 0-.66.23c-.18.16-.27.39-.28.69h-1.93l-.01-.03c-.01-.79.25-1.36.77-1.77c.54-.39 1.24-.59 2.11-.59c.93 0 1.66.23 2.19.68c.54.45.81 1.06.81 1.82c0 .5-.15.91-.44 1.31\"/></svg>",
    "mdi:key-chain": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12.67 13.67c-.47.46-1.04.83-1.67 1.06V23H8v-2H5v-3h3v-3.28c-1.74-.62-3-2.26-3-4.22C5 8 7 6 9.5 6h.1c-.47.95-.68 2-.57 3.08c-.59.2-1.03.76-1.03 1.42c0 .83.67 1.5 1.5 1.5c.23 0 .45-.06.65-.15c.64.84 1.52 1.47 2.52 1.82m8.06 5.77l-2.76 1.16l-.78-1.84l-2.76 1.17l-1.17-2.77L16.03 16l-1.27-3c-1.85.08-3.65-.95-4.41-2.75c-.96-2.29.12-4.93 2.41-5.9c.24-.1.5-.17.74-.23C12.84 2.87 11.5 2 10 2C7.79 2 6 3.79 6 6v.24c-.3.26-.6.58-.85.91C5.06 6.78 5 6.4 5 6c0-2.76 2.24-5 5-5s5 2.24 5 5c0 1.42-.6 2.67-1.55 3.57c.42.43 1.05.56 1.63.31c.77-.32 1.12-1.2.8-1.96a1 1 0 0 0-.14-.26C15.9 7.13 16 6.58 16 6c0-.63-.1-1.24-.28-1.81c1.28.36 2.38 1.25 2.93 2.57c.76 1.8.24 3.81-1.15 5.05zM13 8.6c.37-.41.65-.89.82-1.42c-.54.27-.85.82-.82 1.42\"/></svg>",
    "mdi:lambda": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m6 20l4.16-12.09L9.34 6H8V4h2c.42 0 .78.26.93.63L16.66 18H18v2h-2c-.43 0-.79-.27-.93-.64l-3.74-8.71L8.12 20z\"/></svg>",
    "mdi:linux": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M14.62 8.35c-.42.28-1.75 1.04-1.95 1.19c-.39.31-.75.29-1.14-.01c-.2-.16-1.53-.92-1.95-1.19c-.48-.31-.45-.7.08-.92c1.64-.69 3.28-.64 4.91.03c.49.21.51.6.05.9m7.22 7.28c-.93-2.09-2.2-3.99-3.84-5.66a4.3 4.3 0 0 1-1.06-1.88c-.1-.33-.17-.67-.24-1.01c-.2-.88-.29-1.78-.7-2.61c-.73-1.58-2-2.4-3.84-2.47c-1.81.05-3.16.81-3.95 2.4c-.21.43-.36.88-.46 1.34c-.17.76-.32 1.55-.5 2.32c-.15.65-.45 1.21-.96 1.71c-1.61 1.57-2.9 3.37-3.88 5.35c-.14.29-.28.58-.37.88c-.19.66.29 1.12.99.96c.44-.09.88-.18 1.3-.31c.41-.15.57-.05.67.35c.65 2.15 2.07 3.66 4.24 4.5c4.12 1.56 8.93-.66 9.97-4.58c.07-.27.17-.37.47-.27c.46.14.93.24 1.4.35c.49.09.85-.16.92-.64c.03-.26-.06-.49-.16-.73\"/></svg>",
    "mdi:math-integral": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M11.5 19.1c-.2 1.1-.6 1.9-1.3 2.4s-1.6.6-2.7.4c-.4-.1-1.2-.2-1.5-.4l.5-1.5c.3.1.9.3 1.2.3c1.1.2 1.7-.3 1.9-1.5L12 5.2c.2-1.2.7-2 1.4-2.6c.7-.5 1.7-.7 2.8-.5c.4.1 1.2.2 1.8.5L17.5 4c-.2-.1-.9-.2-1.2-.3c-1.3-.2-2 .4-2.3 1.9z\"/></svg>",
    "mdi:memory": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M17 17H7V7h10m4 4V9h-2V7a2 2 0 0 0-2-2h-2V3h-2v2h-2V3H9v2H7c-1.11 0-2 .89-2 2v2H3v2h2v2H3v2h2v2a2 2 0 0 0 2 2h2v2h2v-2h2v2h2v-2h2a2 2 0 0 0 2-2v-2h2v-2h-2v-2m-6 2h-2v-2h2m2-2H9v6h6z\"/></svg>",
    "mdi:message-fast-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M20 5H9c-1.1 0-2 .9-2 2v14l4-4h9c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2m0 10h-9.8L9 16.2V7h11zM3 7c-.6 0-1 .4-1 1s.4 1 1 1h2V7zm-1 4c-.6 0-1 .4-1 1s.4 1 1 1h3v-2zm-1 4c-.6 0-1 .4-1 1s.4 1 1 1h4v-2z\"/></svg>",
    "mdi:message-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2m0 14H5.2L4 17.2V4h16z\"/></svg>",
    "mdi:microsoft-windows": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M3 12V6.75l6-1.32v6.48zm17-9v8.75l-10 .15V5.21zM3 13l6 .09v6.81l-6-1.15zm17 .25V22l-10-1.91V13.1z\"/></svg>",
    "mdi:ninja": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M7.75 13c-.01-.35.15-.69.42-.92c.75.16 1.45.47 2.08.92c0 .68-.56 1.24-1.25 1.24S7.76 13.69 7.75 13m6 0c.63-.44 1.33-.75 2.08-.91c.27.23.43.57.42.91c0 .7-.56 1.26-1.25 1.26s-1.25-.56-1.25-1.26M12 9c-2.77-.04-5.5.65-7.93 2L4 12c0 1.23.29 2.44.84 3.54a47.6 47.6 0 0 1 14.32 0c.55-1.1.84-2.31.84-3.54l-.07-1A15.85 15.85 0 0 0 12 9m0-7a10 10 0 0 1 10 10a10 10 0 0 1-10 10A10 10 0 0 1 2 12A10 10 0 0 1 12 2\"/></svg>",
    "mdi:palette": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M17.5 12a1.5 1.5 0 0 1-1.5-1.5A1.5 1.5 0 0 1 17.5 9a1.5 1.5 0 0 1 1.5 1.5a1.5 1.5 0 0 1-1.5 1.5m-3-4A1.5 1.5 0 0 1 13 6.5A1.5 1.5 0 0 1 14.5 5A1.5 1.5 0 0 1 16 6.5A1.5 1.5 0 0 1 14.5 8m-5 0A1.5 1.5 0 0 1 8 6.5A1.5 1.5 0 0 1 9.5 5A1.5 1.5 0 0 1 11 6.5A1.5 1.5 0 0 1 9.5 8m-3 4A1.5 1.5 0 0 1 5 10.5A1.5 1.5 0 0 1 6.5 9A1.5 1.5 0 0 1 8 10.5A1.5 1.5 0 0 1 6.5 12M12 3a9 9 0 0 0-9 9a9 9 0 0 0 9 9a1.5 1.5 0 0 0 1.5-1.5c0-.39-.15-.74-.39-1c-.23-.27-.38-.62-.38-1a1.5 1.5 0 0 1 1.5-1.5H16a5 5 0 0 0 5-5c0-4.42-4.03-8-9-8\"/></svg>",
    "mdi:plus-circle-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 20c-4.41 0-8-3.59-8-8s3.59-8 8-8s8 3.59 8 8s-3.59 8-8 8m0-18A10 10 0 0 0 2 12a10 10 0 0 0 10 10a10 10 0 0 0 10-10A10 10 0 0 0 12 2m1 5h-2v4H7v2h4v4h2v-4h4v-2h-4z\"/></svg>",
    "mdi:printer": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M18 3H6v4h12m1 5a1 1 0 0 1-1-1a1 1 0 0 1 1-1a1 1 0 0 1 1 1a1 1 0 0 1-1 1m-3 7H8v-5h8m3-6H5a3 3 0 0 0-3 3v6h4v4h12v-4h4v-6a3 3 0 0 0-3-3\"/></svg>",
    "mdi:regex": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M16 16.92c-.33.05-.66.08-1 .08s-.67-.03-1-.08v-3.51l-2.5 2.48c-.5-.39-1-.89-1.39-1.39l2.48-2.5H9.08c-.05-.33-.08-.66-.08-1s.03-.67.08-1h3.51l-2.48-2.5c.19-.25.39-.5.65-.74c.24-.26.49-.46.74-.65L14 8.59V5.08c.33-.05.66-.08 1-.08s.67.03 1 .08v3.51l2.5-2.48c.5.39 1 .89 1.39 1.39L17.41 10h3.51c.05.33.08.66.08 1s-.03.67-.08 1h-3.51l2.48 2.5c-.19.25-.39.5-.65.74c-.24.26-.49.46-.74.65L16 13.41zM5 19a2 2 0 0 1 2-2a2 2 0 0 1 2 2a2 2 0 0 1-2 2a2 2 0 0 1-2-2\"/></svg>",
    "mdi:scale-balance": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 3c-1.27 0-2.4.8-2.82 2H3v2h1.95L2 14c-.47 2 1 3 3.5 3s4.06-1 3.5-3L6.05 7h3.12c.33.85.98 1.5 1.83 1.83V20H2v2h20v-2h-9V8.82c.85-.32 1.5-.97 1.82-1.82h3.13L15 14c-.47 2 1 3 3.5 3s4.06-1 3.5-3l-2.95-7H21V5h-6.17C14.4 3.8 13.27 3 12 3m0 2a1 1 0 0 1 1 1a1 1 0 0 1-1 1a1 1 0 0 1-1-1a1 1 0 0 1 1-1m-6.5 5.25L7 14H4zm13 0L20 14h-3z\"/></svg>",
    "mdi:shuffle-variant": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m17 3l5.25 4.5L17 12l5.25 4.5L17 21v-3h-2.74l-2.82-2.82l2.12-2.12L15.5 15H17V9h-1.5l-9 9H2v-3h3.26l9-9H17zM2 6h4.5l2.82 2.82l-2.12 2.12L5.26 9H2z\"/></svg>",
    "mdi:star-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m12 15.39l-3.76 2.27l.99-4.28l-3.32-2.88l4.38-.37L12 6.09l1.71 4.04l4.38.37l-3.32 2.88l.99 4.28M22 9.24l-7.19-.61L12 2L9.19 8.63L2 9.24l5.45 4.73L5.82 21L12 17.27L18.18 21l-1.64-7.03z\"/></svg>",
    "mdi:swap-horizontal": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m21 9l-4-4v3h-7v2h7v3M7 11l-4 4l4 4v-3h7v-2H7z\"/></svg>",
    "mdi:text-box": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M14 17H7v-2h7m3-2H7v-2h10m0-2H7V7h10m2-4H5c-1.11 0-2 .89-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2\"/></svg>",
    "mdi:text-box-outline": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M5 3c-1.11 0-2 .89-2 2v14c0 1.11.89 2 2 2h14c1.11 0 2-.89 2-2V5c0-1.11-.89-2-2-2zm0 2h14v14H5zm2 2v2h10V7zm0 4v2h10v-2zm0 4v2h7v-2z\"/></svg>",
    "mdi:weather-night": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m17.75 4.09l-2.53 1.94l.91 3.06l-2.63-1.81l-2.63 1.81l.91-3.06l-2.53-1.94L12.44 4l1.06-3l1.06 3zm3.5 6.91l-1.64 1.25l.59 1.98l-1.7-1.17l-1.7 1.17l.59-1.98L15.75 11l2.06-.05L18.5 9l.69 1.95zm-2.28 4.95c.83-.08 1.72 1.1 1.19 1.85c-.32.45-.66.87-1.08 1.27C15.17 23 8.84 23 4.94 19.07c-3.91-3.9-3.91-10.24 0-14.14c.4-.4.82-.76 1.27-1.08c.75-.53 1.93.36 1.85 1.19c-.27 2.86.69 5.83 2.89 8.02a9.96 9.96 0 0 0 8.02 2.89m-1.64 2.02a12.08 12.08 0 0 1-7.8-3.47c-2.17-2.19-3.33-5-3.49-7.82c-2.81 3.14-2.7 7.96.31 10.98c3.02 3.01 7.84 3.12 10.98.31\"/></svg>",
    "mdi:weather-sunny": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 7a5 5 0 0 1 5 5a5 5 0 0 1-5 5a5 5 0 0 1-5-5a5 5 0 0 1 5-5m0 2a3 3 0 0 0-3 3a3 3 0 0 0 3 3a3 3 0 0 0 3-3a3 3 0 0 0-3-3m0-7l2.39 3.42C13.65 5.15 12.84 5 12 5s-1.65.15-2.39.42zM3.34 7l4.16-.35A7.2 7.2 0 0 0 5.94 8.5c-.44.74-.69 1.5-.83 2.29zm.02 10l1.76-3.77a7.13 7.13 0 0 0 2.38 4.14zM20.65 7l-1.77 3.79a7.02 7.02 0 0 0-2.38-4.15zm-.01 10l-4.14.36c.59-.51 1.12-1.14 1.54-1.86c.42-.73.69-1.5.83-2.29zM12 22l-2.41-3.44c.74.27 1.55.44 2.41.44c.82 0 1.63-.17 2.37-.44z\"/></svg>",
    "mdi:weight": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 3a4 4 0 0 1 4 4c0 .73-.19 1.41-.54 2H18c.95 0 1.75.67 1.95 1.56C21.96 18.57 22 18.78 22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2c0-.22.04-.43 2.05-8.44C4.25 9.67 5.05 9 6 9h2.54A3.9 3.9 0 0 1 8 7a4 4 0 0 1 4-4m0 2a2 2 0 0 0-2 2a2 2 0 0 0 2 2a2 2 0 0 0 2-2a2 2 0 0 0-2-2\"/></svg>",
    "mdi:wikipedia": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m14.97 18.95l-2.56-6.03c-1.02 1.99-2.14 4.08-3.1 6.03c-.01.01-.47 0-.47 0C7.37 15.5 5.85 12.1 4.37 8.68C4.03 7.84 2.83 6.5 2 6.5v-.45h5.06v.45c-.6 0-1.62.4-1.36 1.05c.72 1.54 3.24 7.51 3.93 9.03c.47-.94 1.8-3.42 2.37-4.47c-.45-.88-1.87-4.18-2.29-5c-.32-.54-1.13-.61-1.75-.61c0-.15.01-.25 0-.44l4.46.01v.4c-.61.03-1.18.24-.92.82c.6 1.24.95 2.13 1.5 3.28c.17-.34 1.07-2.19 1.5-3.16c.26-.65-.13-.91-1.21-.91c.01-.12.01-.33.01-.43c1.39-.01 3.48-.01 3.85-.02v.42c-.71.03-1.44.41-1.82.99L13.5 11.3c.18.51 1.96 4.46 2.15 4.9l3.85-8.83c-.3-.72-1.16-.87-1.5-.87v-.45l4 .03v.42c-.88 0-1.43.5-1.75 1.25c-.8 1.79-3.25 7.49-4.85 11.2z\"/></svg>",
    "simple-icons:caddy": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M11.094.47c-.842 0-1.696.092-2.552.288a11.4 11.4 0 0 0-4.87 2.423a10.6 10.6 0 0 0-2.36 2.826A10 10 0 0 0 .305 8.582c-.398 1.62-.4 3.336-.043 5.048c.085.405.183.809.31 1.212a11.9 11.9 0 0 0 1.662 3.729a3 3 0 0 0-.086.427a3.323 3.323 0 0 0 2.848 3.71a3.3 3.3 0 0 0 1.947-.346c1.045.51 2.17.864 3.339 1.04a11.7 11.7 0 0 0 4.285-.155a11.6 11.6 0 0 0 4.936-2.485a10.6 10.6 0 0 0 2.352-2.894a11.2 11.2 0 0 0 1.356-4.424a11.2 11.2 0 0 0-.498-4.335q.263-.116.486-.293h.001c.402-.322.693-.794.777-1.342a2.146 2.146 0 0 0-1.79-2.434a2.1 2.1 0 0 0-1.205.171q-.059-.064-.113-.13a11.7 11.7 0 0 0-3.476-2.93a13 13 0 0 0-1.76-.81a13.6 13.6 0 0 0-2.06-.613a12 12 0 0 0-2.48-.258Zm.714.328a10 10 0 0 1 1.028.042a9.9 9.9 0 0 1 2.743.639c.984.39 1.89.958 2.707 1.632a10.8 10.8 0 0 1 2.091 2.328q.038.06.07.12a2.12 2.12 0 0 0-.435 2.646c-.158.114-.97.692-1.634 1.183c-.414.308-.733.557-.733.557l.581.68s.296-.276.665-.638c.572-.562 1.229-1.233 1.395-1.403a2.12 2.12 0 0 0 1.907.677a11.2 11.2 0 0 1-.013 4.046a11.4 11.4 0 0 1-1.475 3.897a12.3 12.3 0 0 1-2.079 2.587c-1.19 1.125-2.633 2.022-4.306 2.531a10.8 10.8 0 0 1-3.973.484a11 11 0 0 1-3.057-.652a3.3 3.3 0 0 0 1.417-2.294a3.3 3.3 0 0 0-.294-1.842c.18-.162.403-.363.656-.6c1.015-.955 2.353-2.303 2.353-2.303l-.47-.599s-1.63.972-2.801 1.728c-.307.198-.573.378-.777.517a3.3 3.3 0 0 0-1.516-.611a3.33 3.33 0 0 0-3.487 2.017a10 10 0 0 1-.695-1.078A11 11 0 0 1 .728 14.8a10 10 0 0 1-.2-1.212c-.164-1.653.103-3.258.629-4.754a13 13 0 0 1 1.087-2.288c.57-.968 1.248-1.872 2.069-2.656A11 11 0 0 1 11.808.797Zm-.147 3.257a3.84 3.84 0 0 0-3.82 3.82v2.36h-.94c-.751 0-1.377.625-1.377 1.377v3.8h1.46v-3.718h9.354v6.264H10.02v1.46h6.4c.751 0 1.377-.625 1.377-1.377v-6.43a1.39 1.39 0 0 0-1.377-1.377h-.94v-2.36a3.84 3.84 0 0 0-3.82-3.819zm0 1.46a2.37 2.37 0 0 1 2.36 2.36v2.36H9.3v-2.36a2.37 2.37 0 0 1 2.36-2.36zm10.141.392a1.253 1.253 0 0 1 1.296 1.434a1.24 1.24 0 0 1-.453.78c-.266.213-.61.318-.968.264a1.253 1.253 0 0 1-1.045-1.42a1.255 1.255 0 0 1 1.17-1.058M5.384 17.425a2.02 2.02 0 0 1 1.917 1.298c.116.3.159.628.114.967a2.015 2.015 0 0 1-2.249 1.728a2.016 2.016 0 0 1-1.727-2.25a2.02 2.02 0 0 1 1.945-1.743\"/></svg>",
    "simple-icons:clojure": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M11.503 12.216c-.119.259-.251.549-.387.858c-.482 1.092-1.016 2.42-1.21 3.271a5 5 0 0 0-.112 1.096q.001.247.022.514c.682.25 1.417.388 2.186.39a6.4 6.4 0 0 0 2.001-.326a4 4 0 0 1-.418-.441c-.854-1.089-1.329-2.682-2.082-5.362M8.355 6.813A6.35 6.35 0 0 0 5.657 12a6.35 6.35 0 0 0 2.625 5.134c.39-1.622 1.366-3.107 2.83-6.084q-.13-.36-.297-.775c-.406-1.018-.991-2.198-1.513-2.733a4.3 4.3 0 0 0-.947-.729m9.172 12.464c-.84-.105-1.533-.232-2.141-.446A7.625 7.625 0 0 1 4.376 12a7.6 7.6 0 0 1 2.6-5.73a5.6 5.6 0 0 0-1.324-.162c-2.236.02-4.597 1.258-5.58 4.602c-.092.486-.07.854-.07 1.29c0 6.627 5.373 12 12 12c4.059 0 7.643-2.017 9.815-5.101c-1.174.293-2.305.433-3.271.436q-.543 0-1.019-.058m-2.254-2.325c.074.036.242.097.475.163a6.35 6.35 0 0 0 2.6-5.115h-.002a6.354 6.354 0 0 0-6.345-6.345a6.3 6.3 0 0 0-1.992.324c1.289 1.468 1.908 3.566 2.507 5.862l.001.003c.001.002.192.637.518 1.48c.326.842.789 1.885 1.293 2.645c.332.51.697.876.945.983M12.001 0a11.98 11.98 0 0 0-9.752 5.013c1.134-.71 2.291-.967 3.301-.957c1.394.004 2.491.436 3.017.732q.19.11.366.233A7.625 7.625 0 0 1 19.625 12a7.6 7.6 0 0 1-2.268 5.425c.344.038.709.063 1.084.061c1.328 0 2.766-.293 3.842-1.198c.703-.592 1.291-1.458 1.617-2.757q.099-.753.1-1.531c0-6.627-5.371-12-11.999-12\"/></svg>",
    "simple-icons:gnuemacs": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 24C5.448 24 .118 18.617.118 12S5.448 0 12 0s11.882 5.383 11.882 12S18.552 24 12 24M12 .661C5.813.661.779 5.748.779 12S5.813 23.339 12 23.339S23.221 18.253 23.221 12S18.187.661 12 .661M8.03 20.197s.978.069 2.236-.042c.51-.045 2.444-.235 3.891-.552c0 0 1.764-.377 2.707-.725c.987-.364 1.524-.673 1.766-1.11c-.011-.09.074-.408-.381-.599c-1.164-.488-2.514-.4-5.185-.457c-2.962-.102-3.948-.598-4.472-.997c-.503-.405-.25-1.526 1.907-2.513c1.086-.526 5.345-1.496 5.345-1.496c-1.434-.709-4.109-1.955-4.659-2.224c-.482-.236-1.254-.591-1.421-1.021c-.19-.413.448-.768.804-.87c1.147-.331 2.766-.536 4.24-.56c.741-.012.861-.059.861-.059c1.022-.17 1.695-.869 1.414-1.976c-.252-1.13-1.579-1.795-2.84-1.565c-1.188.217-4.05 1.048-4.05 1.048c3.539-.031 4.131.028 4.395.398c.156.218-.071.518-1.015.672c-1.027.168-3.163.37-3.163.37c-2.049.122-3.492.13-3.925 1.046c-.283.599.302 1.129.558 1.46c1.082 1.204 2.646 1.853 3.652 2.331c.379.18 1.49.52 1.49.52c-3.265-.18-5.619.823-7.001 1.977c-1.562 1.445-.871 3.168 2.33 4.228c1.891.626 2.828.921 5.648.667c1.661-.09 1.923-.036 1.939.1c.023.192-1.845.669-2.355.816c-1.298.374-4.699 1.129-4.716 1.133\"/></svg>",
    "simple-icons:markdown": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M22.27 19.385H1.73A1.73 1.73 0 0 1 0 17.655V6.345a1.73 1.73 0 0 1 1.73-1.73h20.54A1.73 1.73 0 0 1 24 6.345v11.308a1.73 1.73 0 0 1-1.73 1.731zM5.769 15.923v-4.5l2.308 2.885l2.307-2.885v4.5h2.308V8.078h-2.308l-2.307 2.885l-2.308-2.885H3.46v7.847zM21.232 12h-2.309V8.077h-2.307V12h-2.308l3.461 4.039z\"/></svg>",
    "simple-icons:nginx": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 0L1.605 6v12L12 24l10.395-6V6zm6 16.59c0 .705-.646 1.29-1.529 1.29c-.631 0-1.351-.255-1.801-.81l-6-7.141v6.66c0 .721-.57 1.29-1.274 1.29H7.32c-.721 0-1.29-.6-1.29-1.29V7.41c0-.705.63-1.29 1.5-1.29c.646 0 1.38.255 1.83.81l5.97 7.141V7.41c0-.721.6-1.29 1.29-1.29h.075c.72 0 1.29.6 1.29 1.29v9.18z\"/></svg>",
    "simple-icons:react": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M14.23 12.004a2.236 2.236 0 0 1-2.235 2.236a2.236 2.236 0 0 1-2.236-2.236a2.236 2.236 0 0 1 2.235-2.236a2.236 2.236 0 0 1 2.236 2.236m2.648-10.69c-1.346 0-3.107.96-4.888 2.622c-1.78-1.653-3.542-2.602-4.887-2.602c-.41 0-.783.093-1.106.278c-1.375.793-1.683 3.264-.973 6.365C1.98 8.917 0 10.42 0 12.004c0 1.59 1.99 3.097 5.043 4.03c-.704 3.113-.39 5.588.988 6.38c.32.187.69.275 1.102.275c1.345 0 3.107-.96 4.888-2.624c1.78 1.654 3.542 2.603 4.887 2.603c.41 0 .783-.09 1.106-.275c1.374-.792 1.683-3.263.973-6.365C22.02 15.096 24 13.59 24 12.004c0-1.59-1.99-3.097-5.043-4.032c.704-3.11.39-5.587-.988-6.38a2.17 2.17 0 0 0-1.092-.278zm-.005 1.09v.006c.225 0 .406.044.558.127c.666.382.955 1.835.73 3.704c-.054.46-.142.945-.25 1.44a23.5 23.5 0 0 0-3.107-.534A24 24 0 0 0 12.769 4.7c1.592-1.48 3.087-2.292 4.105-2.295zm-9.77.02c1.012 0 2.514.808 4.11 2.28c-.686.72-1.37 1.537-2.02 2.442a23 23 0 0 0-3.113.538a15 15 0 0 1-.254-1.42c-.23-1.868.054-3.32.714-3.707c.19-.09.4-.127.563-.132zm4.882 3.05q.684.704 1.36 1.564c-.44-.02-.89-.034-1.345-.034q-.691-.001-1.36.034c.44-.572.895-1.096 1.345-1.565zM12 8.1c.74 0 1.477.034 2.202.093q.61.874 1.183 1.86q.557.961 1.018 1.946c-.308.655-.646 1.31-1.013 1.95c-.38.66-.773 1.288-1.18 1.87a25.6 25.6 0 0 1-4.412.005a27 27 0 0 1-1.183-1.86q-.557-.961-1.018-1.946a25 25 0 0 1 1.013-1.954c.38-.66.773-1.286 1.18-1.868A25 25 0 0 1 12 8.098zm-3.635.254c-.24.377-.48.763-.704 1.16q-.336.585-.635 1.174c-.265-.656-.49-1.31-.676-1.947c.64-.15 1.315-.283 2.015-.386zm7.26 0q1.044.153 2.006.387c-.18.632-.405 1.282-.66 1.933a26 26 0 0 0-1.345-2.32zm3.063.675q.727.226 1.375.498c1.732.74 2.852 1.708 2.852 2.476c-.005.768-1.125 1.74-2.857 2.475c-.42.18-.88.342-1.355.493a24 24 0 0 0-1.1-2.98c.45-1.017.81-2.01 1.085-2.964zm-13.395.004c.278.96.645 1.957 1.1 2.98a23 23 0 0 0-1.086 2.964c-.484-.15-.944-.318-1.37-.5c-1.732-.737-2.852-1.706-2.852-2.474s1.12-1.742 2.852-2.476c.42-.18.88-.342 1.356-.494m11.678 4.28c.265.657.49 1.312.676 1.948c-.64.157-1.316.29-2.016.39a26 26 0 0 0 1.341-2.338zm-9.945.02c.2.392.41.783.64 1.175q.345.586.705 1.143a22 22 0 0 1-2.006-.386c.18-.63.406-1.282.66-1.933zM17.92 16.32c.112.493.2.968.254 1.423c.23 1.868-.054 3.32-.714 3.708c-.147.09-.338.128-.563.128c-1.012 0-2.514-.807-4.11-2.28c.686-.72 1.37-1.536 2.02-2.44c1.107-.118 2.154-.3 3.113-.54zm-11.83.01c.96.234 2.006.415 3.107.532c.66.905 1.345 1.727 2.035 2.446c-1.595 1.483-3.092 2.295-4.11 2.295a1.2 1.2 0 0 1-.553-.132c-.666-.38-.955-1.834-.73-3.703c.054-.46.142-.944.25-1.438zm4.56.64q.661.032 1.345.034q.691.001 1.36-.034c-.44.572-.895 1.095-1.345 1.565q-.684-.706-1.36-1.565\"/></svg>",
    "simple-icons:rescript": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M23.29 1.8q-.45-.6-.9-.9c-1.3-.9-2.899-.9-6.098-.9H7.696C4.498 0 2.9 0 1.8.8c-.4.3-.7.6-1 1C0 2.9 0 4.5 0 7.7v8.6c0 3.2 0 4.8.8 5.9q.45.6.9.9c1.199.9 2.798.9 5.996.9h8.596c3.199 0 4.798 0 5.898-.8q.6-.45.9-.9c.799-1.1.799-2.7.799-5.9V7.7c.2-3.2.2-4.8-.6-5.9ZM11.194 16.5c0 .2 0 .5-.1.8c0 .2-.1.3-.1.5c-.1.1-.2.3-.4.5s-.4.3-.6.4c-.3.1-.7.1-1.399.1c-.8 0-1.1 0-1.4-.1q-.6-.3-.899-.9c-.1-.3-.1-.7-.1-1.4v-8c0-.9 0-1.4.2-1.7s.4-.5.8-.7c.3-.2.8-.2 1.699-.2h2.299zm5.097-4.9a2.794 2.794 0 0 1-2.798-2.8c0-1.6 1.3-2.8 2.798-2.8c1.5 0 2.8 1.3 2.8 2.8s-1.3 2.8-2.8 2.8\"/></svg>",
    "simple-icons:sass": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12 0c6.627 0 12 5.373 12 12s-5.373 12-12 12S0 18.627 0 12S5.373 0 12 0M9.615 15.998c.175.645.156 1.248-.024 1.792l-.065.18q-.037.092-.078.176q-.21.435-.555.81c-.698.759-1.672 1.047-2.09.805c-.45-.262-.226-1.335.584-2.19c.871-.918 2.12-1.509 2.12-1.509v-.003zm9.911-10.861c-.542-2.133-4.077-2.834-7.422-1.645c-1.989.707-4.144 1.818-5.693 3.267C4.568 8.48 4.275 9.98 4.396 10.607c.427 2.211 3.457 3.657 4.703 4.73v.006c-.367.18-3.056 1.529-3.686 2.925c-.675 1.47.105 2.521.615 2.655c1.575.436 3.195-.36 4.065-1.649c.84-1.261.766-2.881.404-3.676c.496-.135 1.08-.195 1.83-.104c2.101.24 2.521 1.56 2.43 2.1c-.09.539-.523.854-.674.944c-.15.091-.195.12-.181.181c.015.09.091.09.21.075c.165-.03 1.096-.45 1.141-1.471c.045-1.29-1.186-2.729-3.375-2.7c-.9.016-1.471.091-1.875.256a.4.4 0 0 0-.105-.105c-1.35-1.455-3.855-2.475-3.75-4.41c.03-.705.285-2.564 4.8-4.814c3.705-1.846 6.661-1.335 7.171-.21c.733 1.604-1.576 4.59-5.431 5.024c-1.47.165-2.235-.404-2.431-.615c-.209-.225-.239-.24-.314-.194c-.12.06-.045.255 0 .375c.12.3.585.825 1.396 1.095c.704.225 2.43.359 4.5-.45c2.324-.899 4.139-3.405 3.614-5.505z\"/></svg>",
    "simple-icons:toml": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M.014 0h5.34v2.652H2.888v18.681h2.468V24H.015zm17.622 5.049v2.78h-4.274v12.935h-3.008V7.83H6.059V5.05zM23.986 24h-5.34v-2.652h2.467V2.667h-2.468V0h5.34v24Z\"/></svg>",
    "simple-icons:typst": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M12.654 17.846q0 1.67.479 2.242q.48.572 1.743.572q1.308 0 3.356-1.319l.871 1.45q-3.835 3.21-6.318 3.209q-2.485 0-3.922-1.187q-1.438-1.23-1.438-4.307V6.989H5.246l-.349-1.626l2.528-.791V2.418L12.654 0v4.835l5.142-.395l-.48 2.857l-4.662-.176z\"/></svg>",
    "simple-icons:webassembly": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"M14.745 0v.129a2.752 2.752 0 1 1-5.504 0V0H0v24h24V0zm-3.291 21.431l-1.169-5.783h-.02l-1.264 5.783H7.39l-1.824-8.497h1.59l1.088 5.783h.02l1.311-5.783h1.487l1.177 5.854h.02l1.242-5.854h1.561l-2.027 8.497zm8.755 0l-.542-1.891h-2.861l-.417 1.891h-1.59l2.056-8.497h2.509l2.5 8.497zm-2.397-6.403l-.694 3.118h2.159l-.796-3.118z\"/></svg>",
    "simple-icons:zig": "<svg  width=\"1em\" height=\"1em\" viewBox=\"0 0 24 24\"><path fill=\"currentColor\" d=\"m23.53 1.02l-7.686 3.45h-7.06l-2.98 3.452h7.173L.47 22.98l7.681-3.607h7.065v-.002l2.978-3.45l-7.148-.001l12.482-14.9zM0 4.47v14.901h1.883l2.98-3.45H3.451v-8h.942l2.824-3.45zm22.117 0l-2.98 3.608h1.412v7.844h-.942l-2.98 3.45H24V4.47z\"/></svg>"
};

// Registry loaded from plugins.json
let registry = null;

// The wasm-bindgen host module (arborium_host.js)
let hostModule = null;

// Cache for loaded grammar plugins
const grammarCache = {};

// Map from grammar handle to plugin instance and session (for window.arboriumHost.parse)
// Each entry: { plugin, language, session }
const handleToPlugin = new Map();
let nextHandle = 1;
let currentHandle = null; // Track the currently active grammar handle

// Cache for fetched sample content
const examplesCache = {};

// Fetch sample content on-demand
async function fetchExample(langId) {
    // Return cached content if available
    if (examplesCache[langId] !== undefined) {
        return examplesCache[langId];
    }

    // Check if sample exists
    const ext = exampleExtensions[langId];
    if (!ext) {
        examplesCache[langId] = null;
        return null;
    }

    // Fetch from server using actual file extension
    try {
        const response = await fetch(`/samples/${langId}.${ext}`);
        if (!response.ok) {
            examplesCache[langId] = null;
            return null;
        }
        const content = await response.text();
        examplesCache[langId] = content;
        return content;
    } catch (e) {
        examplesCache[langId] = null;
        return null;
    }
}

// Load a grammar plugin on demand (used internally by plugin-provider)
async function loadGrammarPlugin(langId) {
    // Return cached plugin if available
    if (grammarCache[langId]) {
        return grammarCache[langId];
    }

    // Find in registry
    const entry = registry?.entries?.find(e => e.language === langId);
    if (!entry) {
        return null; // Language not available
    }

    // Determine paths based on dev mode
    const jsPath = registry.dev_mode ? entry.local_js : entry.cdn_js;
    const wasmPath = registry.dev_mode ? entry.local_wasm : entry.cdn_wasm;

    try {
        // Import the wasm-bindgen generated JS module and initialize it.
        //
        // Plugins are built by `cargo xtask build` using wasm-bindgen `--target web`,
        // so the generated `grammar.js` exports a default init function and the
        // plugin API as named exports.
        const module = await import(jsPath);
        await module.default(wasmPath);

        grammarCache[langId] = module;
        return module;
    } catch (e) {
        console.error(`Failed to load grammar '${langId}':`, e);
        return null;
    }
}

// Setup window.arboriumHost interface for the Rust host to call into
function setupArboriumHost() {
    // Expose loadGrammar for testing/debugging
    window.loadGrammar = loadGrammarPlugin;

    window.arboriumHost = {
        // Check if a language is available (sync check)
        isLanguageAvailable(language) {
            const entry = registry?.entries?.find(e => e.language === language);
            return !!entry || !!grammarCache[language];
        },

        // Load a grammar and return a handle (async)
        async loadGrammar(language) {
            const plugin = await loadGrammarPlugin(language);
            if (!plugin) return 0; // 0 = not found

            // Create new handle with a persistent session for incremental highlighting
            const handle = nextHandle++;
            const session = plugin.create_session();
            handleToPlugin.set(handle, { plugin, language, session });
            currentHandle = handle; // Track this as the current active handle
            return handle;
        },

        // Parse text using a grammar handle (sync)
        // Uses persistent session for incremental highlighting
        parse(handle, text) {
            const entry = handleToPlugin.get(handle);
            if (!entry) return { spans: [], injections: [] };

            const { plugin, session } = entry;
            try {
                // Reuse the session - just set new text and parse
                plugin.set_text(session, text);
                const val = plugin.parse(session);
                return { spans: val?.spans || [], injections: val?.injections || [] };
            } catch (e) {
                console.error(`Parse error for handle ${handle}:`, e);
                return { spans: [], injections: [] };
            }
        },

        // Free a grammar handle and release its session memory
        freeGrammar(handle) {
            const entry = handleToPlugin.get(handle);
            if (!entry) return;

            const { plugin, session } = entry;
            try {
                // Free the session
                if (plugin.free_session) {
                    plugin.free_session(session);
                }
            } catch (e) {
                console.error(`Error freeing session for handle ${handle}:`, e);
            }
            handleToPlugin.delete(handle);
        },
    };
}

// Load the wasm-bindgen host module
async function loadHost() {
    if (hostModule) return hostModule;

    // Setup the interface the host imports
    setupArboriumHost();

    try {
        // Import and initialize the wasm-bindgen host
        const module = await import('/pkg/arborium_host.js');
        await module.default('/pkg/arborium_host_bg.wasm');

        hostModule = {
            highlight: module.highlight,
            isLanguageAvailable: module.isLanguageAvailable,
        };
        return hostModule;
    } catch (e) {
        console.error('Failed to load arborium host:', e);
        throw e;
    }
}

// Highlight code using the Rust host (handles injections automatically)
async function highlightCode(langId, source) {
    const host = await loadHost();
    return host.highlight(langId, source);
}

// Escape HTML special characters (used for non-highlighted display)
function escapeHtml(text) {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#x27;');
}

let wasmLoaded = false;
let allLanguages = [];
let selectedLang = null;
let highlightedIndex = 0;

// Language picker elements
const langPicker = document.getElementById('lang-picker');
const langLabel = document.getElementById('lang-label');
const langInput = document.getElementById('lang-input');
const langDropdown = document.getElementById('lang-dropdown');

// Get icon SVG for a language
function getIconSvg(id) {
    const info = languageInfo[id];
    const iconName = info?.icon;
    if (iconName && icons[iconName]) {
        return icons[iconName];
    }
    // Fallback icon
    return icons['mdi:code-tags'] || '';
}

// Fuzzy match function
function fuzzyMatch(pattern, text) {
    pattern = pattern.toLowerCase();
    text = text.toLowerCase();

    let patternIdx = 0;
    let textIdx = 0;
    let matchPositions = [];

    while (patternIdx < pattern.length && textIdx < text.length) {
        if (pattern[patternIdx] === text[textIdx]) {
            matchPositions.push(textIdx);
            patternIdx++;
        }
        textIdx++;
    }

    return patternIdx === pattern.length ? matchPositions : null;
}

// Highlight matched characters
function highlightMatches(text, positions) {
    if (!positions || positions.length === 0) return text;

    let result = '';
    let lastPos = 0;

    for (const pos of positions) {
        result += text.slice(lastPos, pos);
        result += `<span class="match-highlight">${text[pos]}</span>`;
        lastPos = pos + 1;
    }
    result += text.slice(lastPos);

    return result;
}

// Filter and render languages
function filterLanguages(query) {
    const filtered = [];

    for (const id of allLanguages) {
        const info = languageInfo[id] || { name: id, tag: 'code' };
        const name = info.name;
        const tag = info.tag;
        const aliases = info.aliases || [];
        const tier = info.tier ?? 99; // Default to low priority if no tier

        // Match against id, name, tag, or aliases
        const idMatch = fuzzyMatch(query, id);
        const nameMatch = fuzzyMatch(query, name);
        const tagMatch = query && tag.toLowerCase().includes(query.toLowerCase());
        // Check aliases - find best alias match
        let aliasMatch = null;
        for (const alias of aliases) {
            const match = fuzzyMatch(query, alias);
            if (match && (!aliasMatch || match.length < aliasMatch.length)) {
                aliasMatch = match;
            }
        }

        if (!query || idMatch || nameMatch || tagMatch || aliasMatch) {
            // Calculate score - prefer exact alias matches
            let score = 200;
            if (nameMatch) score = nameMatch.length;
            else if (aliasMatch) score = aliasMatch.length + 50; // Slightly deprioritize aliases
            else if (idMatch) score = idMatch.length + 100;

            filtered.push({
                id,
                name,
                tag,
                tier,
                aliases,
                idMatch,
                nameMatch,
                aliasMatch,
                score
            });
        }
    }

    // Sort by tier first (lower is better), then by match quality/name
    filtered.sort((a, b) => {
        // When searching (query exists), prioritize match quality
        if (query) {
            return a.score - b.score;
        }
        // When browsing (no query), sort by tier then name
        if (a.tier !== b.tier) {
            return a.tier - b.tier;
        }
        return a.name.localeCompare(b.name);
    });

    return filtered;
}

// Render dropdown - sorted by tier (lower = more popular)
function renderDropdown(languages, query = '') {
    if (languages.length === 0) {
        langDropdown.innerHTML = '<div class="lang-dropdown-empty">No languages found</div>';
        return;
    }

    langDropdown.innerHTML = languages.map((lang, idx) => {
        const info = languageInfo[lang.id] || { name: lang.id, tag: 'code' };
        const nameHtml = lang.nameMatch
            ? highlightMatches(info.name, lang.nameMatch)
            : info.name;

        const isSelected = selectedLang === lang.id;
        const isHighlighted = idx === highlightedIndex;
        const iconSvg = getIconSvg(lang.id);

        return `
            <div class="lang-option ${isSelected ? 'selected' : ''} ${isHighlighted ? 'highlighted' : ''}"
                 data-id="${lang.id}" data-index="${idx}">
                <span class="lang-icon">${iconSvg}</span>
                <span class="lang-name">${nameHtml}</span>
            </div>
        `;
    }).join('');
}

// Update the rich label display (no tag in toolbar - keep controls clean)
function updateLabel(id) {
    const info = languageInfo[id] || { name: id, tag: 'code' };
    const iconSvg = getIconSvg(id);
    // Get the caret element to preserve it
    const caretEl = langLabel.querySelector('.picker-caret');
    const caretHtml = caretEl ? caretEl.outerHTML : '';
    langLabel.innerHTML = `
        <span class="lang-icon">${iconSvg}</span>
        <span class="lang-name">${info.name}</span>
        ${caretHtml}
    `;

    // Update language info panel
    updateLangInfoPanel(id);

    // Update watermark
    updateWatermark(id);
}

// Update watermark icon in editor
function updateWatermark(id) {
    let watermark = document.querySelector('.lang-watermark');
    if (!watermark) {
        watermark = document.createElement('div');
        watermark.className = 'lang-watermark';
        document.querySelector('.panel').appendChild(watermark);
    }
    watermark.innerHTML = getIconSvg(id);
}

// Update language info panel with metadata (marginalia style - museum label)
function updateLangInfoPanel(id) {
    const info = languageInfo[id];
    const panel = document.getElementById('lang-info-panel');
    if (!panel || !info) return;

    // Only show panel if we have interesting metadata
    if (!info.description && !info.trivia && !info.inventor) {
        panel.classList.remove('visible');
        return;
    }

    // Build attribution line: "2020, Author Name" or just "Author Name" or just "2020"
    let attribution = '';
    if (info.year && info.inventor) {
        attribution = `${info.year}, ${info.inventor}`;
    } else if (info.year) {
        attribution = `${info.year}`;
    } else if (info.inventor) {
        attribution = info.inventor;
    }

    // "About [Language]" heading - link if URL available
    const linkUrl = info.url || info.wikipedia;
    const headingText = `About ${info.name}`;
    const nameHtml = linkUrl
        ? `<a class="lang-name-link" href="${linkUrl}" target="_blank" rel="noopener">${headingText}</a>`
        : `<span class="lang-name">${headingText}</span>`;

    // Update sample bar (separate from panel)
    const sampleBar = document.getElementById('sample-bar');
    if (info.sample && sampleBar) {
        const s = info.sample;
        // Parse org/repo from URL like https://github.com/org/repo/...
        let repoLabel = 'Source';
        if (s.link) {
            try {
                const url = new URL(s.link);
                const parts = url.pathname.split('/').filter(Boolean);
                if (parts.length >= 2) {
                    repoLabel = `${parts[0]}/${parts[1]}`;
                }
            } catch (e) {
                // Keep default
            }
        }
        const gitIcon = icons['mdi:git'] || icons['mdi:source-branch'] || '';
        const scalesIcon = icons['mdi:scale-balance'] || '';
        const descText = s.description || 'Sample code';
        sampleBar.innerHTML = `
            <span class="sample-desc" title="${descText}">${descText}</span>
            ${s.license ? `<span class="sample-license"><span class="sample-license-icon">${scalesIcon}</span>${s.license}</span>` : ''}
            ${s.link ? `<a class="sample-link" href="${s.link}" target="_blank" rel="noopener"><span class="sample-icon">${gitIcon}</span>${repoLabel}</a>` : ''}
        `;
        sampleBar.classList.add('visible');
    } else if (sampleBar) {
        sampleBar.classList.remove('visible');
    }

    panel.innerHTML = `
        <div class="card-header">
            ${nameHtml}
        </div>
        ${attribution ? `<div class="card-attribution">${attribution}</div>` : ''}
        ${info.description ? `<div class="card-body"><p class="card-description">${info.description}</p></div>` : ''}
        ${info.trivia ? `<div class="card-trivia">${info.trivia}</div>` : ''}
    `;
    panel.classList.add('visible');
}

// Enter search mode
function enterSearchMode() {
    langPicker.classList.add('searching');
    langInput.value = '';
    const filtered = filterLanguages('');
    // Start from currently selected language, not 0
    if (selectedLang) {
        const selectedIndex = filtered.findIndex(l => l.id === selectedLang);
        highlightedIndex = selectedIndex >= 0 ? selectedIndex : 0;
    } else {
        highlightedIndex = 0;
    }
    renderDropdown(filtered);
    langDropdown.classList.add('open');
    langInput.focus();
    // Scroll to show the selected item
    scrollToHighlighted();
}

// Exit search mode
function exitSearchMode() {
    langPicker.classList.remove('searching');
    langDropdown.classList.remove('open');
    langInput.blur();
}

// Preview a language (without committing selection)
async function previewLanguage(id) {
    // Load example if available and re-highlight
    const sourceEl = document.getElementById('source');
    const editorContainer = document.getElementById('demo');
    const loadingMessage = document.getElementById('loading-message');
    const example = await fetchExample(id);
    if (example) {
        sourceEl.value = example;
    }
    // Temporarily highlight with this language
    if (wasmLoaded) {
        const source = sourceEl.value;
        const output = document.getElementById('output');
        if (source) {
            // Update loading message with pretty language name
            const langInfo = languageInfo[id];
            const prettyName = langInfo?.name || id;
            if (loadingMessage) {
                loadingMessage.textContent = `Loading ${prettyName} grammar`;
            }

            // Delay showing loading state by 500ms
            const loadingTimeout = setTimeout(() => {
                editorContainer.classList.add('highlighting');
            }, 500);
            try {
                const html = await highlightCode(id, source);
                output.innerHTML = html;
            } catch (e) {
                console.error('Preview highlighting failed:', e);
            } finally {
                clearTimeout(loadingTimeout);
                editorContainer.classList.remove('highlighting');
            }
        }
    }
    // Update info panel and watermark during preview
    updateLangInfoPanel(id);
    updateWatermark(id);
}

// Select a language
async function selectLanguage(id) {
    // Free the previous grammar's session before loading a new one
    if (currentHandle !== null && window.arboriumHost && window.arboriumHost.freeGrammar) {
        window.arboriumHost.freeGrammar(currentHandle);
        currentHandle = null;
    }

    selectedLang = id;
    updateLabel(id);
    exitSearchMode();

    // Update URL hash
    history.replaceState(null, '', `#${id}`);

    // Load example if available
    const example = await fetchExample(id);
    if (example) {
        document.getElementById('source').value = example;
    }

    doHighlight();
}

// Event handlers
langLabel.addEventListener('click', () => {
    enterSearchMode();
});

langInput.addEventListener('input', () => {
    highlightedIndex = 0;
    const query = langInput.value;
    const filtered = filterLanguages(query);
    renderDropdown(filtered, query);
});

langInput.addEventListener('keydown', (e) => {
    const query = langInput.value;
    const filtered = filterLanguages(query);

    if (e.key === 'ArrowDown') {
        e.preventDefault();
        highlightedIndex = Math.min(highlightedIndex + 1, filtered.length - 1);
        renderDropdown(filtered, query);
        scrollToHighlighted();
        // Preview the highlighted language
        if (filtered[highlightedIndex]) {
            previewLanguage(filtered[highlightedIndex].id);
        }
    } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        highlightedIndex = Math.max(highlightedIndex - 1, 0);
        renderDropdown(filtered, query);
        scrollToHighlighted();
        // Preview the highlighted language
        if (filtered[highlightedIndex]) {
            previewLanguage(filtered[highlightedIndex].id);
        }
    } else if (e.key === 'Enter') {
        e.preventDefault();
        if (highlightedIndex >= 0 && highlightedIndex < filtered.length) {
            selectLanguage(filtered[highlightedIndex].id);
        } else if (filtered.length > 0) {
            selectLanguage(filtered[0].id);
        }
    } else if (e.key === 'Escape') {
        // Restore the previously selected language
        if (selectedLang) {
            previewLanguage(selectedLang);
        }
        exitSearchMode();
    }
});

langInput.addEventListener('blur', () => {
    // Small delay to allow click events on dropdown to fire
    setTimeout(() => {
        if (!langDropdown.matches(':hover')) {
            // Restore the previously selected language
            if (selectedLang) {
                previewLanguage(selectedLang);
            }
            exitSearchMode();
        }
    }, 150);
});

function scrollToHighlighted() {
    const highlighted = langDropdown.querySelector('.highlighted');
    if (highlighted) {
        highlighted.scrollIntoView({ block: 'nearest' });
    }
}

langDropdown.addEventListener('click', (e) => {
    const option = e.target.closest('.lang-option');
    if (option) {
        selectLanguage(option.dataset.id);
    }
});

langDropdown.addEventListener('mouseover', (e) => {
    const option = e.target.closest('.lang-option');
    if (option) {
        // Update highlighting without re-rendering (to preserve icons)
        const newIndex = parseInt(option.dataset.index, 10);
        if (newIndex !== highlightedIndex) {
            // Remove old highlight
            const oldHighlighted = langDropdown.querySelector('.highlighted');
            if (oldHighlighted) oldHighlighted.classList.remove('highlighted');
            // Add new highlight
            option.classList.add('highlighted');
            highlightedIndex = newIndex;
        }
        // Don't preview on hover - too noisy. Only preview on keyboard nav.
    }
});

// Theme metadata: id -> { name, variant } - generated from arborium-theme
const themeInfo = {
    "catppuccin-mocha": { name: "Catppuccin Mocha", variant: "dark" },
    "catppuccin-latte": { name: "Catppuccin Latte", variant: "light" },
    "catppuccin-frappe": { name: "Catppuccin Frappé", variant: "dark" },
    "catppuccin-macchiato": { name: "Catppuccin Macchiato", variant: "dark" },
    "dracula": { name: "Dracula", variant: "dark" },
    "tokyo-night": { name: "Tokyo Night", variant: "dark" },
    "nord": { name: "Nord", variant: "dark" },
    "one-dark": { name: "One Dark", variant: "dark" },
    "github-dark": { name: "GitHub Dark", variant: "dark" },
    "github-light": { name: "GitHub Light", variant: "light" },
    "gruvbox-dark": { name: "Gruvbox Dark", variant: "dark" },
    "gruvbox-light": { name: "Gruvbox Light", variant: "light" },
    "monokai": { name: "Monokai", variant: "dark" },
    "kanagawa-dragon": { name: "Kanagawa Dragon", variant: "dark" },
    "rose-pine-moon": { name: "Rosé Pine Moon", variant: "dark" },
    "ayu-dark": { name: "Ayu Dark", variant: "dark" },
    "ayu-light": { name: "Ayu Light", variant: "light" },
    "solarized-dark": { name: "Solarized Dark", variant: "dark" },
    "solarized-light": { name: "Solarized Light", variant: "light" },
    "ef-melissa-dark": { name: "EF Melissa Dark", variant: "dark" },
    "melange-dark": { name: "Melange Dark", variant: "dark" },
    "melange-light": { name: "Melange Light", variant: "light" },
    "light-owl": { name: "Light Owl", variant: "light" },
    "lucius-light": { name: "Lucius Light", variant: "light" },
    "rustdoc-light": { name: "Rustdoc Light", variant: "light" },
    "rustdoc-dark": { name: "Rustdoc Dark", variant: "dark" },
    "rustdoc-ayu": { name: "Rustdoc Ayu", variant: "dark" },
    "dayfox": { name: "Dayfox", variant: "light" },
    "alabaster": { name: "Alabaster", variant: "light" },
    "cobalt2": { name: "Cobalt2", variant: "dark" },
    "zenburn": { name: "Zenburn", variant: "dark" },
    "desert256": { name: "Desert256", variant: "dark" }
};

const allThemes = Object.keys(themeInfo);
let selectedTheme = null;
let themeHighlightedIndex = 0;

// Mode toggle (dark/light filter)
let currentMode = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
const modeDarkBtn = document.getElementById('mode-dark');
const modeLightBtn = document.getElementById('mode-light');

// Build theme pairs dynamically: try to find a matching light/dark counterpart
function findThemePair(themeId) {
    const theme = themeInfo[themeId];
    if (!theme) return null;

    const targetVariant = theme.variant === 'dark' ? 'light' : 'dark';

    // Try to find a theme with similar name but opposite variant
    // e.g., "gruvbox-dark" -> "gruvbox-light", "github-dark" -> "github-light"
    const baseName = themeId.replace(/-dark$/, '').replace(/-light$/, '');
    const pairedId = baseName + '-' + targetVariant;
    if (themeInfo[pairedId] && themeInfo[pairedId].variant === targetVariant) {
        return pairedId;
    }

    // Special cases for catppuccin (mocha/macchiato/frappe are dark, latte is light)
    if (themeId.startsWith('catppuccin-')) {
        if (theme.variant === 'dark') {
            return 'catppuccin-latte';
        } else {
            return 'catppuccin-mocha';
        }
    }

    // Fallback: first theme of opposite variant
    return allThemes.find(id => themeInfo[id].variant === targetVariant) || null;
}

function setMode(mode) {
    currentMode = mode;
    modeDarkBtn.classList.toggle('active', mode === 'dark');
    modeLightBtn.classList.toggle('active', mode === 'light');
    localStorage.setItem('arborium-mode', mode);

    // If current theme doesn't match mode, switch to paired theme or fallback
    if (selectedTheme && themeInfo[selectedTheme].variant !== mode) {
        const pairedTheme = findThemePair(selectedTheme);
        if (pairedTheme) {
            selectTheme(pairedTheme);
        }
    }

    // Sync swatch filter in Theme support section
    const swatches = document.querySelector('.theme-swatches');
    if (swatches) {
        swatches.dataset.showMode = mode;
        document.getElementById('swatch-mode-dark')?.classList.toggle('active', mode === 'dark');
        document.getElementById('swatch-mode-light')?.classList.toggle('active', mode === 'light');
    }
}

modeDarkBtn.addEventListener('click', () => setMode('dark'));
modeLightBtn.addEventListener('click', () => setMode('light'));

// Swatch mode toggle (for Theme support section)
const swatchModeToggle = document.getElementById('swatch-mode-toggle');
const swatchModeDark = document.getElementById('swatch-mode-dark');
const swatchModeLight = document.getElementById('swatch-mode-light');
const themeSwatches = document.querySelector('.theme-swatches');

function setSwatchMode(mode) {
    if (themeSwatches) {
        themeSwatches.dataset.showMode = mode;
    }
    if (swatchModeDark && swatchModeLight) {
        swatchModeDark.classList.toggle('active', mode === 'dark');
        swatchModeLight.classList.toggle('active', mode === 'light');
    }
}

if (swatchModeDark) {
    swatchModeDark.addEventListener('click', () => setSwatchMode('dark'));
}
if (swatchModeLight) {
    swatchModeLight.addEventListener('click', () => setSwatchMode('light'));
}

// Sync swatch mode with main mode on load
setSwatchMode(currentMode);

// Theme picker elements
const themePicker = document.getElementById('theme-picker');
const themeLabel = document.getElementById('theme-label');
const themeInput = document.getElementById('theme-input');
const themeDropdown = document.getElementById('theme-dropdown');

// Filter and render themes
function filterThemes(query) {
    const filtered = [];

    for (const id of allThemes) {
        const info = themeInfo[id];
        const name = info.name;
        const variant = info.variant;

        // Filter by current mode (dark/light)
        if (variant !== currentMode) continue;

        // Match against id, name, or variant
        const idMatch = fuzzyMatch(query, id);
        const nameMatch = fuzzyMatch(query, name);
        const variantMatch = query && variant.toLowerCase().includes(query.toLowerCase());

        if (!query || idMatch || nameMatch || variantMatch) {
            filtered.push({
                id,
                name,
                variant,
                idMatch,
                nameMatch,
                score: nameMatch ? nameMatch.length : (idMatch ? idMatch.length + 100 : 200)
            });
        }
    }

    // Sort by match quality
    filtered.sort((a, b) => a.score - b.score);

    return filtered;
}

// Render theme dropdown
function renderThemeDropdown(themes) {
    if (themes.length === 0) {
        themeDropdown.innerHTML = '<div class="theme-dropdown-empty">No themes found</div>';
        return;
    }

    const moonIcon = icons['mdi:weather-night'] || '';
    const sunIcon = icons['mdi:weather-sunny'] || '';

    themeDropdown.innerHTML = themes.map((theme, idx) => {
        const nameHtml = theme.nameMatch
            ? highlightMatches(theme.name, theme.nameMatch)
            : theme.name;

        const isSelected = selectedTheme === theme.id;
        const isHighlighted = idx === themeHighlightedIndex;
        const variantIcon = theme.variant === 'dark' ? moonIcon : sunIcon;

        return `
            <div class="theme-option ${isSelected ? 'selected' : ''} ${isHighlighted ? 'highlighted' : ''}"
                 data-id="${theme.id}" data-index="${idx}">
                <span class="theme-name-text">${nameHtml}</span>
                <span class="theme-variant-icon">${variantIcon}</span>
            </div>
        `;
    }).join('');
}

// Update theme label display (no tag in toolbar - keep controls clean)
function updateThemeLabel(id) {
    const info = themeInfo[id];
    // Get the caret element to preserve it
    const caretEl = themeLabel.querySelector('.picker-caret');
    const caretHtml = caretEl ? caretEl.outerHTML : '';
    themeLabel.innerHTML = `
        <span class="theme-name">${info.name}</span>
        ${caretHtml}
    `;
}

// Enter theme search mode
function enterThemeSearchMode() {
    themePicker.classList.add('searching');
    themeInput.value = '';
    themeHighlightedIndex = 0;
    const filtered = filterThemes('');
    renderThemeDropdown(filtered);
    themeDropdown.classList.add('open');
    themeInput.focus();
}

// Exit theme search mode
function exitThemeSearchMode() {
    themePicker.classList.remove('searching');
    themeDropdown.classList.remove('open');
    themeInput.blur();
}

// Preview a theme (without committing selection)
function previewTheme(id) {
    document.documentElement.dataset.theme = id;
}

// Select a theme
function selectTheme(id) {
    selectedTheme = id;
    updateThemeLabel(id);
    exitThemeSearchMode();
    document.documentElement.dataset.theme = id;
    localStorage.setItem('arborium-theme', id);
}

// Theme event handlers
themeLabel.addEventListener('click', () => {
    enterThemeSearchMode();
});

themeInput.addEventListener('input', () => {
    themeHighlightedIndex = 0;
    const filtered = filterThemes(themeInput.value);
    renderThemeDropdown(filtered);
});

themeInput.addEventListener('keydown', (e) => {
    const filtered = filterThemes(themeInput.value);

    if (e.key === 'ArrowDown') {
        e.preventDefault();
        themeHighlightedIndex = Math.min(themeHighlightedIndex + 1, filtered.length - 1);
        renderThemeDropdown(filtered);
        scrollToThemeHighlighted();
        // Preview the highlighted theme
        if (filtered[themeHighlightedIndex]) {
            previewTheme(filtered[themeHighlightedIndex].id);
        }
    } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        themeHighlightedIndex = Math.max(themeHighlightedIndex - 1, 0);
        renderThemeDropdown(filtered);
        scrollToThemeHighlighted();
        // Preview the highlighted theme
        if (filtered[themeHighlightedIndex]) {
            previewTheme(filtered[themeHighlightedIndex].id);
        }
    } else if (e.key === 'Enter') {
        e.preventDefault();
        if (themeHighlightedIndex >= 0 && themeHighlightedIndex < filtered.length) {
            selectTheme(filtered[themeHighlightedIndex].id);
        } else if (filtered.length > 0) {
            selectTheme(filtered[0].id);
        }
    } else if (e.key === 'Escape') {
        // Restore the previously selected theme
        if (selectedTheme) {
            previewTheme(selectedTheme);
        }
        exitThemeSearchMode();
    }
});

themeInput.addEventListener('blur', () => {
    setTimeout(() => {
        if (!themeDropdown.matches(':hover')) {
            // Restore the previously selected theme
            if (selectedTheme) {
                previewTheme(selectedTheme);
            }
            exitThemeSearchMode();
        }
    }, 150);
});

function scrollToThemeHighlighted() {
    const highlighted = themeDropdown.querySelector('.highlighted');
    if (highlighted) {
        highlighted.scrollIntoView({ block: 'nearest' });
    }
}

themeDropdown.addEventListener('click', (e) => {
    const option = e.target.closest('.theme-option');
    if (option) {
        selectTheme(option.dataset.id);
    }
});

themeDropdown.addEventListener('mouseover', (e) => {
    const option = e.target.closest('.theme-option');
    if (option) {
        // Update highlighting without re-rendering
        const newIndex = parseInt(option.dataset.index, 10);
        if (newIndex !== themeHighlightedIndex) {
            // Remove old highlight
            const oldHighlighted = themeDropdown.querySelector('.highlighted');
            if (oldHighlighted) oldHighlighted.classList.remove('highlighted');
            // Add new highlight
            option.classList.add('highlighted');
            themeHighlightedIndex = newIndex;
        }
        // Don't preview on hover - too noisy. Only preview on keyboard nav.
    }
});

// Initialize mode and theme
const savedMode = localStorage.getItem('arborium-mode');
const savedTheme = localStorage.getItem('arborium-theme');

// Set mode first (from saved, or from system preference)
if (savedMode) {
    setMode(savedMode);
} else {
    setMode(currentMode); // Uses system preference detected earlier
}

// Then set theme (if saved theme matches current mode, use it; otherwise use default for mode)
if (savedTheme && themeInfo[savedTheme] && themeInfo[savedTheme].variant === currentMode) {
    selectTheme(savedTheme);
} else {
    // Pick first theme matching current mode
    const defaultTheme = currentMode === 'dark' ? 'catppuccin-mocha' : 'catppuccin-latte';
    selectTheme(defaultTheme);
}

async function initialize() {
    try {
        // Load the plugins manifest
        const pluginsResponse = await fetch('/plugins.json');
        if (!pluginsResponse.ok) {
            throw new Error(`Failed to load plugins: ${pluginsResponse.status}`);
        }
        registry = await pluginsResponse.json();

        // Get list of available languages from registry
        allLanguages = registry.entries.map(e => e.language);

        // Sort by tier (lower is better), then by name
        allLanguages.sort((a, b) => {
            const infoA = languageInfo[a] || { name: a };
            const infoB = languageInfo[b] || { name: b };
            const tierA = infoA.tier ?? 99;
            const tierB = infoB.tier ?? 99;
            if (tierA !== tierB) {
                return tierA - tierB;
            }
            return (infoA.name || a).localeCompare(infoB.name || b);
        });

        wasmLoaded = true;

        // Check URL hash for language selection
        const hashLang = window.location.hash.slice(1);
        if (hashLang && allLanguages.includes(hashLang)) {
            selectLanguage(hashLang);
        } else if (allLanguages.includes('rust')) {
            selectLanguage('rust');
        } else if (allLanguages.length > 0) {
            selectLanguage(allLanguages[0]);
        }

    } catch (error) {
        console.error('Failed to initialize:', error);
        document.getElementById('output').innerHTML = `<span class="error">Failed to initialize: ${error}</span>`;
        updateStatus('Failed to load registry', false);
    }
}

async function doHighlight() {
    if (!wasmLoaded || !selectedLang) return;

    const source = document.getElementById('source').value;
    const output = document.getElementById('output');
    const editorContainer = document.getElementById('demo');
    const loadingMessage = document.getElementById('loading-message');

    if (!source) {
        output.innerHTML = '<span class="loading">Enter some code to highlight</span>';
        return;
    }

    // Update loading message with pretty language name
    const langInfo = languageInfo[selectedLang];
    const prettyName = langInfo?.name || selectedLang;
    if (loadingMessage) {
        loadingMessage.textContent = `Loading ${prettyName} grammar`;
    }

    // Delay showing loading state by 500ms - only show if highlighting takes longer
    const loadingTimeout = setTimeout(() => {
        editorContainer.classList.add('highlighting');
    }, 500);

    try {
        const start = performance.now();
        const html = await highlightCode(selectedLang, source);
        const elapsed = (performance.now() - start).toFixed(2);

        output.innerHTML = html;
        updateStatus(`Highlighted ${source.length} chars in ${elapsed}ms`, true);
    } catch (error) {
        console.error('Highlighting failed:', error);
        output.innerHTML = `<span class="error">${error}</span>`;
        updateStatus('Highlighting failed', false);
    } finally {
        clearTimeout(loadingTimeout);
        editorContainer.classList.remove('highlighting');
    }
}

function updateStatus(message, success) {
    const status = document.getElementById('status');
    status.textContent = message;
    status.className = success ? 'status success' : 'status';
}

document.getElementById('source').addEventListener('input', doHighlight);

// Global keyboard shortcuts
document.addEventListener('keydown', (e) => {
    const activeEl = document.activeElement;
    const isInTextarea = activeEl.tagName === 'TEXTAREA';
    const isInInput = activeEl.tagName === 'INPUT';

    // Cmd/Ctrl+K opens language picker from anywhere
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        enterSearchMode();
        return;
    }

    // "/" key opens language picker (if not in an input)
    if (e.key === '/' && !isInTextarea && !isInInput) {
        e.preventDefault();
        enterSearchMode();
        return;
    }
});

// Populate language marquee
function populateLangMarquee() {
    const marquee = document.getElementById('lang-marquee');
    if (!marquee) return;

    // Get all language names sorted alphabetically
    const langNames = Object.entries(languageInfo)
        .map(([id, info]) => ({ id, name: info.name || id, icon: info.icon }))
        .sort((a, b) => a.name.localeCompare(b.name));

    // Create items - duplicate the list for seamless scrolling
    const createItems = () => langNames.map(lang => {
        const iconSvg = getIconSvg(lang.id);
        return `<span class="lang-marquee-item">${iconSvg}${lang.name}</span>`;
    }).join('');

    // Duplicate content for seamless loop
    marquee.innerHTML = createItems() + createItems();

    // Update lang count
    const langCount = document.getElementById('lang-count');
    if (langCount) {
        langCount.textContent = langNames.length;
    }

    // Update tagline
    const tagline = document.getElementById('tagline');
    if (tagline) {
        tagline.textContent = `— regex hater club`;
    }
}

// Random button - randomize language and theme together
const randomBtn = document.getElementById('random-btn');
if (randomBtn) {
    randomBtn.addEventListener('click', () => {
        if (!wasmLoaded || allLanguages.length === 0) return;

        // Pick a random language
        const randomLangIndex = Math.floor(Math.random() * allLanguages.length);
        const randomLang = allLanguages[randomLangIndex];

        // Pick a random theme that matches current mode
        const themesForMode = allThemes.filter(id => themeInfo[id].variant === currentMode);
        const randomThemeIndex = Math.floor(Math.random() * themesForMode.length);
        const randomTheme = themesForMode[randomThemeIndex];

        // Apply both with a subtle animation effect
        randomBtn.style.transform = 'rotate(180deg)';
        setTimeout(() => {
            randomBtn.style.transform = '';
        }, 300);

        selectLanguage(randomLang);
        selectTheme(randomTheme);
    });
}

initialize();
populateLangMarquee();