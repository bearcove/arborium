# Adding Language Grammars to Arborium

This guide explains how to add support for new programming languages to Arborium.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Step-by-Step Guide](#step-by-step-guide)
4. [Directory Structure](#directory-structure)
5. [The arborium.yaml File](#the-arboriumyaml-file)
6. [Language Groups](#language-groups)
7. [Grammar Files](#grammar-files)
8. [Query Files](#query-files)
9. [Sample Files](#sample-files)
10. [Building and Testing](#building-and-testing)
11. [Troubleshooting](#troubleshooting)
12. [Examples](#examples)

---

## Overview

Arborium uses [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) grammars to provide syntax highlighting and language support. Adding a new language involves:

1. **Finding or creating** a Tree-sitter grammar for your language
2. **Creating a language definition directory** with the required files
3. **Writing an `arborium.yaml` configuration** file (single source of truth)
4. **Running the build system** to generate Rust crate files
5. **Testing** your language support

The entire process is automated through the `cargo xtask` build system, which reads your configuration and generates all necessary Rust code.

---

## Prerequisites

Before adding a language, you need:

- [ ] **A Tree-sitter grammar** for your language
  - Check https://github.com/tree-sitter for official grammars
  - Verify the grammar is maintained and works with current Tree-sitter versions
- [ ] **Sample code files** in your target language for testing
- [ ] **License information** for the grammar repository
- [ ] **Language metadata** (inventor, year created, description, etc.)

---

## Step-by-Step Guide

### 1. Choose the Appropriate Language Group

Languages are organized into semantic groups based on their characteristics:

| Group | Purpose | Examples |
|-------|---------|----------|
| `group-acorn` | Web languages | JavaScript, TypeScript, HTML, CSS |
| `group-birch` | Systems languages | C, C++, Zig, Assembly |
| `group-cedar` | JVM languages | Java, Kotlin, Scala, Clojure |
| `group-fern` | Functional languages | Haskell, Elm, OCaml, Erlang |
| `group-hazel` | Scripting languages | Python, Ruby, Bash, Perl, PHP |
| `group-maple` | Data/Config languages | SQL, YAML, TOML, GraphQL |
| `group-moss` | Scientific languages | Julia, MATLAB, R |
| `group-pine` | Modern languages | Rust, Swift, Dart, WIT |
| `group-sage` | .NET languages | C#, F#, Visual Basic |
| `group-willow` | Markup/Documentation | Markdown, HTML, XML, SVG |

**Example:** Apache Groovy would go in `group-cedar` because it's a JVM language.

### 2. Create the Language Directory Structure

Create the following structure in your chosen group:

```
langs/group-{name}/{language}/def/
├── arborium.yaml          # Configuration file (REQUIRED)
├── grammar/               # Grammar source files
│   ├── grammar.js         # Tree-sitter grammar definition (REQUIRED)
│   └── scanner.c          # External scanner (if needed by grammar)
├── queries/               # Tree-sitter query files
│   ├── highlights.scm     # Syntax highlighting rules (REQUIRED)
│   ├── injections.scm     # Language injection rules (optional)
│   └── locals.scm         # Scope/symbol tracking (optional)
└── samples/               # Example code files
    └── example.{ext}      # At least one sample file (REQUIRED)
```

**Example for Groovy:**
```bash
mkdir -p langs/group-cedar/groovy/def/{grammar,queries,samples}
```

### 3. Copy Grammar Files from Upstream

Find the Tree-sitter grammar repository and copy the necessary files:

```bash
# Clone the upstream grammar repo (temporarily)
git clone https://github.com/tree-sitter/tree-sitter-groovy /tmp/tree-sitter-groovy

# Copy grammar files
cp /tmp/tree-sitter-groovy/grammar.js langs/group-cedar/groovy/def/grammar/
cp /tmp/tree-sitter-groovy/src/scanner.c langs/group-cedar/groovy/def/grammar/  # if exists

# Copy query files
cp /tmp/tree-sitter-groovy/queries/highlights.scm langs/group-cedar/groovy/def/queries/
cp /tmp/tree-sitter-groovy/queries/injections.scm langs/group-cedar/groovy/def/queries/  # if exists

# Copy or create sample files
cp /tmp/tree-sitter-groovy/examples/* langs/group-cedar/groovy/def/samples/
```

### 4. Create the `arborium.yaml` Configuration File

This is the **single source of truth** for your language. The build system reads this file to generate all Rust code automatically.

**Template:**

```yaml
repo: https://github.com/tree-sitter/tree-sitter-groovy
commit: abc123...  # Full commit hash from the grammar repo
license: MIT       # License of the grammar repository

grammars:
  - id: groovy                    # Unique identifier (lowercase, no spaces)
    name: Groovy                  # Display name
    tag: code                     # Category: "code", "data", "markup", "config"
    tier: 2                       # Quality tier: 1 (best) to 5 (experimental)

    # Technical flags
    has_scanner: true             # Set to true if scanner.c exists
    generate_component: true      # Set to true to include in WASM builds

    # Visual representation
    icon: devicon-plain:groovy    # Icon identifier (from iconify.design)

    # Metadata (REQUIRED for documentation)
    inventor: James Strachan
    year: 2003
    description: "Apache Groovy is a dynamic language for the JVM with syntax similar to Python and Ruby, featuring optional static typing and metaprogramming capabilities."
    link: https://en.wikipedia.org/wiki/Apache_Groovy
    trivia: "Groovy was created in 2003 by James Strachan and became an Apache project in 2015. It powers the Gradle build system and is widely used for DSLs."

    # Sample files (at least one REQUIRED)
    samples:
      - path: samples/example.groovy
        description: Basic Groovy syntax demonstrating closures and dynamic typing
        link: https://github.com/tree-sitter/tree-sitter-groovy/blob/main/examples/example.groovy
        license: Apache-2.0

      # Additional samples (optional)
      - path: samples/DSL.groovy
        description: Groovy DSL example showing builder pattern
        link: https://github.com/tree-sitter/tree-sitter-groovy/blob/main/examples/DSL.groovy
        license: Apache-2.0
```

### 5. Field Reference for `arborium.yaml`

#### Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `repo` | String | ✅ Yes | URL of the Tree-sitter grammar repository |
| `commit` | String | ✅ Yes | Full commit hash from the grammar repo (for reproducibility) |
| `license` | String | ✅ Yes | SPDX license identifier (e.g., "MIT", "Apache-2.0") |

#### Grammar Fields (in `grammars` list)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | String | ✅ Yes | Unique identifier (lowercase, use hyphens for multi-word) |
| `name` | String | ✅ Yes | Human-readable display name |
| `tag` | String | ✅ Yes | Category: `"code"`, `"data"`, `"markup"`, `"config"` |
| `tier` | Integer | ✅ Yes | Quality tier (1-5): 1=production, 2=stable, 3=good, 4=usable, 5=experimental |
| `icon` | String | ✅ Yes | Icon identifier from [Iconify](https://iconify.design) |
| `inventor` | String | ✅ Yes | Person or organization that created the language |
| `year` | Integer | ✅ Yes | Year the language was created |
| `description` | String | ✅ Yes | 1-2 sentence description of the language |
| `link` | String | ✅ Yes | Wikipedia or official documentation URL |
| `trivia` | String | ✅ Yes | Interesting fact about the language |
| `has_scanner` | Boolean | No | Set to `true` if `scanner.c` exists (default: `false`) |
| `generate_component` | Boolean | No | Set to `true` to include in WASM plugin builds (default: `false`) |
| `grammar_path` | String | No | For multi-grammar crates (e.g., `"dtd"` for XML/DTD) |

#### Sample Fields (in `samples` list)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | String | ✅ Yes | Relative path to sample file (from `def/` directory) |
| `description` | String | ✅ Yes | Brief description of what the sample demonstrates |
| `link` | String | No | URL to original source of the sample code |
| `license` | String | No | License of the sample code (if different from grammar) |

### 6. Understanding Tier Levels

Choose the appropriate tier based on grammar quality and maturity:

| Tier | Quality | Use When |
|------|---------|----------|
| 1 | Production | Official grammar, excellent highlighting, widely used |
| 2 | Stable | Community grammar, good highlighting, tested |
| 3 | Good | Working grammar, acceptable highlighting, some edge cases |
| 4 | Usable | Basic grammar, limited testing, known issues |
| 5 | Experimental | New/incomplete grammar, proof-of-concept |

**Examples:**
- **Tier 1**: Rust, JavaScript, Python, TypeScript
- **Tier 2**: Kotlin, Go, Swift, Elixir
- **Tier 3**: WIT, Thrift, Dockerfile, Nix
- **Tier 4**: Rarely used languages or very new grammars
- **Tier 5**: Experimental or unfinished grammars

### 7. Generate the Language Crate

Run the build system to generate all Rust code automatically:

```bash
# Generate crate for a single language
cargo xtask gen groovy

# Generate with version bump (for updates)
cargo xtask gen groovy --version patch

# Dry-run to preview changes
cargo xtask gen groovy --dry-run

# Generate all languages
cargo xtask gen
```

This will create:
```
langs/group-cedar/groovy/crate/
├── Cargo.toml          # Auto-generated package manifest
├── build.rs            # Auto-generated build script
├── src/
│   └── lib.rs          # Auto-generated Rust bindings
└── grammar/
    └── src/
        ├── parser.c    # Auto-generated from grammar.js
        ├── scanner.c   # Copied from def/grammar/ if exists
        ├── grammar.json
        └── node-types.json
```

**⚠️ Important:** Never manually edit files in the `crate/` directory! They are auto-generated from `arborium.yaml`.

### 8. Validate Your Configuration

Run the linter to check for errors:

```bash
# Standard validation
cargo xtask lint

# Strict validation (enforces all best practices)
cargo xtask lint --strict
```

Common validation checks:
- ✅ All required fields present in `arborium.yaml`
- ✅ Sample files actually exist
- ✅ Tier is between 1 and 5
- ✅ Icon identifier is valid
- ✅ Commit hash is full (not abbreviated)
- ✅ License is valid SPDX identifier

### 9. Build and Test

Build the language to ensure it compiles:

```bash
# Build a specific language
cargo xtask build groovy

# Build all languages
cargo xtask build

# Run the demo server (includes all languages)
cargo xtask serve

# Development mode (faster rebuilds)
cargo xtask serve --dev
```

The demo server will be available at `http://localhost:8080` where you can test your language's syntax highlighting.

### 10. Test Your Language

1. **Open the demo** at http://localhost:8080
2. **Find your language** in the language selector
3. **Paste sample code** or select one of your sample files
4. **Verify syntax highlighting** works correctly
5. **Test edge cases** (strings, comments, operators, etc.)

---

## Directory Structure

### Complete Example: Kotlin Language

```
langs/group-cedar/kotlin/
├── def/                                    # Source files (you edit these)
│   ├── arborium.yaml                      # Configuration (EDIT THIS)
│   ├── grammar/
│   │   ├── grammar.js                     # From tree-sitter-kotlin
│   │   └── scanner.c                      # From tree-sitter-kotlin
│   ├── queries/
│   │   ├── highlights.scm                 # Syntax highlighting rules
│   │   └── injections.scm                 # Optional injections
│   └── samples/
│       └── Sequences.kt                   # Example Kotlin code
└── crate/                                  # Generated files (DO NOT EDIT)
    ├── Cargo.toml                         # Auto-generated
    ├── build.rs                           # Auto-generated
    ├── src/
    │   └── lib.rs                         # Auto-generated
    └── grammar/
        └── src/
            ├── parser.c                   # Auto-generated from grammar.js
            ├── scanner.c                  # Copied from def/grammar/
            ├── grammar.json               # Auto-generated
            └── node-types.json            # Auto-generated
```

### What Goes Where?

| Directory | Purpose | Edit? |
|-----------|---------|-------|
| `def/` | Source files you maintain | ✅ YES |
| `def/arborium.yaml` | Single source of truth | ✅ YES |
| `def/grammar/` | Tree-sitter grammar files from upstream | ✅ YES (when updating) |
| `def/queries/` | Syntax highlighting queries from upstream | ✅ YES (when updating) |
| `def/samples/` | Example code for testing | ✅ YES |
| `crate/` | Auto-generated Rust crate | ❌ NO (regenerated by `cargo xtask gen`) |

---

## The arborium.yaml File

The `arborium.yaml` file is the **single source of truth** for your language configuration. The build system reads this file and generates all Rust code automatically.

### Real-World Example: WIT Language

```yaml
repo: https://github.com/bytecodealliance/tree-sitter-wit
commit: a80c1f47baa5bfb13e2b5f49aa5304e3dab94948
license: Apache-2.0 WITH LLVM-exception

grammars:
  - id: wit
    name: WIT
    tag: data
    tier: 3
    has_scanner: true
    generate_component: true
    icon: simple-icons:webassembly

    inventor: Bytecode Alliance
    year: 2021
    description: "WebAssembly Interface Types (WIT) is an IDL for defining component interfaces in the WebAssembly Component Model."
    link: https://component-model.bytecodealliance.org/design/wit.html
    trivia: "WIT is part of the WebAssembly Component Model initiative to enable language-agnostic component composition."

    samples:
      - path: samples/example.wit
        description: WIT interface showing records, variants, enums, and functions
        link: https://github.com/bytecodealliance/tree-sitter-wit/blob/main/examples/example.wit
        license: Apache-2.0 WITH LLVM-exception
```

### Configuration Tips

1. **Get the commit hash:**
   ```bash
   cd /tmp/tree-sitter-groovy
   git rev-parse HEAD  # Copy this full hash
   ```

2. **Find an appropriate icon:**
   - Visit https://iconify.design
   - Search for your language (e.g., "groovy", "java", "python")
   - Use format: `collection:icon-name` (e.g., `devicon-plain:groovy`)

3. **Write a good description:**
   - Keep it 1-2 sentences
   - Focus on key characteristics
   - Mention the runtime/platform if relevant
   - Example: "Apache Groovy is a dynamic language for the JVM with syntax similar to Python and Ruby, featuring optional static typing and metaprogramming capabilities."

4. **Find interesting trivia:**
   - Language creator's motivation
   - Historical significance
   - Notable systems built with it
   - Interesting technical features

---

## Language Groups

### Choosing the Right Group

The group determines where your language appears in the repository structure and how it's categorized. Choose based on the language's primary characteristics:

#### `group-cedar` (JVM Languages)

**Use when:** Your language runs on the Java Virtual Machine

**Examples:**
- Java
- Kotlin
- Scala
- Clojure
- **Groovy** ← Perfect fit!

**Rationale:** JVM languages share runtime characteristics, interoperability features, and common tooling ecosystems.

#### `group-hazel` (Scripting Languages)

**Use when:** Your language is primarily used for scripting, automation, or glue code

**Examples:**
- Python
- Ruby
- Bash
- Perl
- PHP
- Lua

**Rationale:** Dynamic languages with interpreted execution and flexible typing.

#### `group-pine` (Modern Languages)

**Use when:** Your language was designed recently with modern language features

**Examples:**
- Rust
- Swift
- Dart
- WIT (WebAssembly Interface Types)

**Rationale:** Languages created in the 2010s-2020s with contemporary design philosophy.

### Group Organization Philosophy

Groups are **semantic**, not strict technical boundaries:

- ✅ **DO** choose based on primary use case and language family
- ✅ **DO** consider developer expectations (where would users look?)
- ❌ **DON'T** stress over edge cases (some languages fit multiple groups)
- ❌ **DON'T** create new groups without discussion

**Edge Cases:**
- **Groovy:** Could be `group-cedar` (JVM) or `group-hazel` (scripting) → Choose `group-cedar` because JVM interop is primary use case
- **TypeScript:** Could be `group-acorn` (web) or `group-pine` (modern) → Choose `group-acorn` because web development is primary use case

---

## Grammar Files

### `grammar.js` - The Tree-sitter Grammar

This file defines the language's syntax using Tree-sitter's JavaScript DSL. You typically **copy this from the upstream repository** rather than writing it yourself.

**Example snippet from Kotlin grammar:**

```javascript
module.exports = grammar({
  name: 'kotlin',

  externals: $ => [
    $.automatic_semicolon,
    $.safe_nav,
    // ... more tokens
  ],

  rules: {
    source_file: $ => repeat($._statement),

    _statement: $ => choice(
      $.property_declaration,
      $.function_declaration,
      $.class_declaration,
      // ... more choices
    ),

    // ... hundreds more rules
  }
});
```

**You don't need to understand Tree-sitter grammar syntax to add a language!** Just copy the file from the upstream repository.

### `scanner.c` - External Scanner (Optional)

Some grammars require a C scanner for complex tokenization that can't be expressed in `grammar.js`. Examples:

- **String interpolation** (Kotlin, JavaScript)
- **Indentation-based syntax** (Python, YAML)
- **Context-sensitive parsing** (C++ templates)

**When to include:**
- ✅ Set `has_scanner: true` in `arborium.yaml` if `scanner.c` exists
- ✅ Copy `scanner.c` from upstream repository
- ❌ Don't try to write your own scanner (requires deep Tree-sitter knowledge)

**Example languages with scanners:**
- Kotlin (`has-scanner #true`)
- WIT (`has-scanner #true`)
- Python (for indentation)
- YAML (for indentation)

---

## Query Files

Tree-sitter queries use the **S-expression** format to pattern-match syntax tree nodes. These files determine how your language is highlighted and how language features are recognized.

### `highlights.scm` - Syntax Highlighting (REQUIRED)

This file maps syntax tree nodes to highlight groups.

**Example from Kotlin:**

```scheme
; Keywords
[
  "fun"
  "class"
  "interface"
  "object"
  "val"
  "var"
] @keyword

; Functions
(function_declaration
  (simple_identifier) @function)

; Strings
(string_literal) @string

; Comments
(line_comment) @comment
(multiline_comment) @comment

; Operators
[
  "+"
  "-"
  "*"
  "/"
  "="
  "=="
  "!="
] @operator

; Types
(type_identifier) @type

; Constants
(boolean_literal) @constant.builtin
(integer_literal) @number
```

**Common highlight groups:**

| Group | Purpose | Example |
|-------|---------|---------|
| `@keyword` | Language keywords | `fun`, `class`, `if`, `return` |
| `@function` | Function names | `myFunction`, `println` |
| `@type` | Type names | `String`, `Int`, `List` |
| `@variable` | Variable names | `count`, `userName` |
| `@string` | String literals | `"hello"`, `'world'` |
| `@number` | Numeric literals | `42`, `3.14` |
| `@comment` | Comments | `// comment`, `/* block */` |
| `@operator` | Operators | `+`, `-`, `==`, `&&` |
| `@constant.builtin` | Built-in constants | `true`, `false`, `null` |
| `@property` | Object properties | `obj.property` |
| `@parameter` | Function parameters | `func(param)` |

**Tips for writing highlights.scm:**

1. **Start with the upstream file** from the Tree-sitter grammar repository
2. **Test thoroughly** - paste various code samples to verify highlighting
3. **Check edge cases** - nested strings, multi-line comments, escape sequences
4. **Use consistent groups** - follow conventions from other languages in Arborium

### `injections.scm` - Language Injection (Optional)

Injections allow one language to be embedded in another. Common use cases:

- **JavaScript in HTML** (`<script>` tags)
- **CSS in HTML** (`<style>` tags)
- **SQL in Python/Ruby** (string literals with SQL queries)
- **Regex patterns** (in string literals)

**Example from TypeScript:**

```scheme
; Highlight template literal strings with embedded expressions
((template_string) @injection.content
  (#set! injection.language "javascript"))

; Highlight JSX/TSX
((jsx_element) @injection.content
  (#set! injection.language "jsx"))
```

**Example from WIT:**

```scheme
; Inject documentation comments as markdown
((comment) @injection.content
  (#set! injection.language "markdown"))
```

**When to include:**
- ✅ Your language embeds other languages (HTML, JSX, SQL)
- ✅ Your language has special string literals (regex, templates)
- ❌ Simple languages without embedding don't need this file

### `locals.scm` - Scope Tracking (Optional)

Defines scopes, definitions, and references for semantic features like:

- **Go to definition**
- **Find references**
- **Renaming**
- **Symbol highlighting**

**Most languages don't need this file for basic syntax highlighting.** It's primarily used for IDE features.

---

## Sample Files

Sample files demonstrate your language's syntax and are used for:

1. **Testing** syntax highlighting during development
2. **Demo** - shown in the web interface for users to try
3. **Documentation** - examples of language features

### Requirements

- ✅ **At least one sample file** is REQUIRED
- ✅ **Valid syntax** - must parse correctly
- ✅ **Representative code** - show common language features
- ✅ **Properly licensed** - know the source and license

### Good Sample File Characteristics

**DO:**
- ✅ Show diverse syntax (keywords, strings, comments, operators)
- ✅ Include typical language idioms
- ✅ Keep it readable (50-200 lines ideal)
- ✅ Add a descriptive header comment
- ✅ Document the source in `arborium.yaml`

**DON'T:**
- ❌ Use massive files (>500 lines)
- ❌ Include generated code
- ❌ Copy proprietary code
- ❌ Use only trivial examples (`print("hello")`)

### Example: Groovy Sample Files

**`samples/example.groovy`** - Basic syntax:
```groovy
// Basic Groovy features
class Person {
    String name
    int age

    def greet() {
        println "Hello, I'm ${name} and I'm ${age} years old"
    }
}

// Closures
def numbers = [1, 2, 3, 4, 5]
def doubled = numbers.collect { it * 2 }

// DSL-style builders
def person = new Person(name: 'Alice', age: 30)
person.greet()
```

**`samples/DSL.groovy`** - Advanced features:
```groovy
// Groovy builder pattern
class EmailBuilder {
    String to
    String subject
    String body

    def to(String recipient) { to = recipient; this }
    def subject(String subj) { subject = subj; this }
    def body(String content) { body = content; this }

    def send() {
        println "Sending email to ${to}: ${subject}"
    }
}

// Using the DSL
new EmailBuilder()
    .to('user@example.com')
    .subject('Hello from Groovy')
    .body('This is a test email')
    .send()
```

### Referencing Samples in arborium.yaml

```yaml
samples:
  - path: samples/example.groovy
    description: Basic Groovy syntax demonstrating classes, closures, and string interpolation
    link: https://github.com/tree-sitter/tree-sitter-groovy/blob/main/examples/basic.groovy
    license: Apache-2.0

  - path: samples/DSL.groovy
    description: Groovy builder pattern showing DSL capabilities
    link: https://github.com/tree-sitter/tree-sitter-groovy/blob/main/examples/dsl.groovy
    license: Apache-2.0
```

---

## Building and Testing

### Build System Overview

Arborium uses a custom build system called `xtask` (located in `/xtask/`). All commands are run through Cargo:

```bash
cargo xtask <command> [options]
```

### Available Commands

#### `cargo xtask gen [language]`

Generates Rust crate files from `arborium.yaml` configuration.

```bash
# Generate a single language
cargo xtask gen groovy

# Generate all languages
cargo xtask gen

# Preview changes without writing files
cargo xtask gen groovy --dry-run

# Bump version during generation
cargo xtask gen groovy --version patch   # 0.1.0 → 0.1.1
cargo xtask gen groovy --version minor   # 0.1.0 → 0.2.0
cargo xtask gen groovy --version major   # 0.1.0 → 1.0.0
```

**What it does:**
1. Parses your `arborium.yaml` file
2. Validates all fields and references
3. Runs `tree-sitter generate` to create parser
4. Generates `Cargo.toml`, `build.rs`, `src/lib.rs`
5. Updates feature flags in `/crates/arborium/Cargo.toml`
6. Updates demo registry in `/demo/registry.json`

**Output:**
```
✓ Loaded registry (97 languages)
✓ Validated groovy configuration
✓ Generated grammar files
✓ Created crate files
✓ Updated feature flags
✓ Updated demo registry
```

#### `cargo xtask lint [--strict]`

Validates all language configurations.

```bash
# Standard validation
cargo xtask lint

# Strict mode (enforces all best practices)
cargo xtask lint --strict
```

**Checks performed:**
- ✅ Required fields present
- ✅ Sample files exist
- ✅ Tier is valid (1-5)
- ✅ Commit hash format
- ✅ License is valid SPDX
- ✅ Icon identifier format
- ✅ No duplicate language IDs
- ✅ Grammar files exist

#### `cargo xtask build [languages...]`

Builds language crates into WASM plugins (if `generate-component #true`).

```bash
# Build a specific language
cargo xtask build groovy

# Build multiple languages
cargo xtask build groovy kotlin scala

# Build all languages
cargo xtask build

# Build with optimizations
cargo xtask build --release
```

#### `cargo xtask serve [--dev]`

Starts a local web server for testing syntax highlighting.

```bash
# Production mode (optimized build)
cargo xtask serve

# Development mode (faster rebuilds, no optimizations)
cargo xtask serve --dev
```

**Access at:** http://localhost:8080

**Features:**
- Language selector with search
- Sample file loading
- Custom code input
- Syntax highlighting preview
- Export highlighted HTML

### Testing Workflow

**Recommended workflow for adding a language:**

```bash
# 1. Create language structure
mkdir -p langs/group-cedar/groovy/def/{grammar,queries,samples}

# 2. Add files (arborium.yaml, grammar files, samples)
# ... (copy files from upstream repo)

# 3. Generate crate
cargo xtask gen groovy

# 4. Validate configuration
cargo xtask lint

# 5. Build (if generate-component is enabled)
cargo xtask build groovy

# 6. Start dev server
cargo xtask serve --dev

# 7. Test in browser at http://localhost:8080
# - Select "Groovy" from language dropdown
# - Load sample files
# - Verify syntax highlighting

# 8. Iterate as needed
# - Edit queries/highlights.scm to improve highlighting
# - Re-run: cargo xtask gen groovy
# - Refresh browser to see changes
```

### Common Build Issues

#### "Grammar file not found"

```
Error: grammar.js not found in langs/group-cedar/groovy/def/grammar/
```

**Fix:** Ensure `grammar.js` exists in the correct location.

#### "Sample file not found"

```
Error: Sample file samples/example.groovy does not exist
```

**Fix:** Ensure sample files referenced in `arborium.yaml` actually exist.

#### "Tree-sitter generate failed"

```
Error: Failed to generate grammar: ...
```

**Fix:** The `grammar.js` file may be invalid. Test it independently:
```bash
cd langs/group-cedar/groovy/def/grammar
tree-sitter generate
tree-sitter test  # if tests exist
```

#### "Scanner not found but has-scanner is true"

```
Error: has-scanner is true but scanner.c not found
```

**Fix:** Either set `has_scanner: false` or copy `scanner.c` from upstream.

---

## Troubleshooting

### Validation Errors

#### Missing Required Fields

```
Error: Missing required field 'inventor' in groovy grammar
```

**Fix:** Add all required fields to `arborium.yaml`:
- `id`, `name`, `tag`, `tier`, `icon`, `aliases`
- `inventor`, `year`, `description`, `link`, `trivia`

#### Invalid Tier

```
Error: Tier must be between 1 and 5 (got 6)
```

**Fix:** Use tier 1-5 only:
- 1 = Production
- 2 = Stable
- 3 = Good
- 4 = Usable
- 5 = Experimental

#### Sample File Missing

```
Error: Sample file samples/example.groovy does not exist
```

**Fix:** Create the file or update the path in `arborium.yaml`.

### Highlighting Issues

#### No Highlighting at All

**Possible causes:**
1. Missing `highlights.scm` file
2. Empty or invalid `highlights.scm`
3. Grammar parsing errors

**Debug steps:**
```bash
# Check if grammar compiles
cd langs/group-cedar/groovy/def/grammar
tree-sitter generate

# Test parsing
tree-sitter parse ../samples/example.groovy

# Check for query errors
tree-sitter highlight ../samples/example.groovy \
  --scope source.groovy \
  --query ../queries/highlights.scm
```

#### Wrong Colors/Groups

**Problem:** Keywords highlighted as variables, etc.

**Fix:** Update `queries/highlights.scm`:
1. Study the parse tree: `tree-sitter parse file.groovy`
2. Find the correct node names
3. Add patterns to `highlights.scm`
4. Regenerate: `cargo xtask gen groovy`
5. Test in browser

**Example parse tree:**
```
(source_file
  (function_declaration
    name: (simple_identifier)    ; ← Target this for @function
    parameters: (parameter_list)
    body: (block)))
```

**Corresponding highlight:**
```scheme
(function_declaration
  name: (simple_identifier) @function)
```

### Build Errors

#### Compiler Errors in Generated Code

**Problem:** C compiler fails on `parser.c` or `scanner.c`

**Likely cause:** Incompatible Tree-sitter grammar version

**Fix:**
1. Check Tree-sitter version: `tree-sitter --version`
2. Verify grammar is compatible (check grammar's README)
3. Try updating Tree-sitter: `npm install -g tree-sitter-cli`
4. Regenerate grammar: `cargo xtask gen groovy`

#### Feature Flag Conflicts

**Problem:** Duplicate feature names or import errors

**Likely cause:** Language ID conflicts with existing language

**Fix:**
1. Choose a unique `id` in `arborium.yaml`
2. Regenerate: `cargo xtask gen groovy`
3. Check `/crates/arborium/Cargo.toml` for conflicts

### Performance Issues

#### Slow Parsing

**Problem:** Large files take too long to parse

**Causes:**
- Grammar is too complex
- External scanner is inefficient
- Ambiguous grammar rules

**Mitigation:**
1. Use a well-maintained grammar (official or popular community version)
2. Report issues to upstream grammar maintainers
3. Consider setting lower tier (3-5) to indicate limitations

#### Large WASM Size

**Problem:** WASM plugin is very large (>1MB)

**Fix:**
- Ensure release mode: `cargo xtask build --release`
- Consider setting `generate-component #false` for rarely-used languages
- Report to upstream if grammar seems bloated

---

## Examples

### Example 1: Adding Apache Groovy

**Step-by-step walkthrough:**

```bash
# 1. Create directory structure
mkdir -p langs/group-cedar/groovy/def/{grammar,queries,samples}

# 2. Clone upstream grammar
git clone https://github.com/tree-sitter/tree-sitter-groovy /tmp/tree-sitter-groovy
cd /tmp/tree-sitter-groovy

# 3. Get commit hash
COMMIT=$(git rev-parse HEAD)
echo $COMMIT  # Copy this

# 4. Copy grammar files
cp grammar.js ~/arborium/langs/group-cedar/groovy/def/grammar/
cp src/scanner.c ~/arborium/langs/group-cedar/groovy/def/grammar/  # if exists

# 5. Copy query files
cp queries/highlights.scm ~/arborium/langs/group-cedar/groovy/def/queries/
cp queries/injections.scm ~/arborium/langs/group-cedar/groovy/def/queries/  # if exists

# 6. Copy sample files
cp examples/*.groovy ~/arborium/langs/group-cedar/groovy/def/samples/

# 7. Create arborium.yaml
cd ~/arborium/langs/group-cedar/groovy/def
cat > arborium.yaml << 'EOF'
repo: https://github.com/tree-sitter/tree-sitter-groovy
commit: PASTE_COMMIT_HASH_HERE
license: MIT

grammars:
  - id: groovy
    name: Groovy
    tag: code
    tier: 2
    has_scanner: true
    generate_component: true
    icon: devicon-plain:groovy

    inventor: James Strachan
    year: 2003
    description: "Apache Groovy is a dynamic language for the JVM with syntax similar to Python and Ruby, featuring optional static typing and metaprogramming capabilities."
    link: https://en.wikipedia.org/wiki/Apache_Groovy
    trivia: "Groovy was created in 2003 by James Strachan and became an Apache project in 2015. It powers the Gradle build system."

    samples:
      - path: samples/example.groovy
        description: Basic Groovy syntax with closures and dynamic typing
        license: Apache-2.0
EOF

# 8. Edit and replace commit hash
# (Replace "PASTE_COMMIT_HASH_HERE" with the actual commit hash)

# 9. Generate crate
cd ~/arborium
cargo xtask gen groovy

# 10. Validate
cargo xtask lint

# 11. Build
cargo xtask build groovy

# 12. Test
cargo xtask serve --dev
# Open http://localhost:8080 and select Groovy

# 13. Commit
git add langs/group-cedar/groovy/
git commit -m "Add Apache Groovy language support"
```

### Example 2: Updating an Existing Grammar

**Scenario:** Update Kotlin to a newer Tree-sitter grammar version

```bash
# 1. Clone latest grammar
git clone https://github.com/fwcd/tree-sitter-kotlin /tmp/tree-sitter-kotlin
cd /tmp/tree-sitter-kotlin
COMMIT=$(git rev-parse HEAD)

# 2. Update grammar files
cp grammar.js ~/arborium/langs/group-cedar/kotlin/def/grammar/
cp src/scanner.c ~/arborium/langs/group-cedar/kotlin/def/grammar/
cp queries/highlights.scm ~/arborium/langs/group-cedar/kotlin/def/queries/

# 3. Update arborium.yaml
cd ~/arborium/langs/group-cedar/kotlin/def
# Edit arborium.yaml: update the 'commit' field with $COMMIT

# 4. Regenerate with version bump
cd ~/arborium
cargo xtask gen kotlin --version patch

# 5. Validate and test
cargo xtask lint
cargo xtask build kotlin
cargo xtask serve --dev
```

### Example 3: Language Without Scanner

**Scenario:** Add a simple language (e.g., TOML) without external scanner

```yaml
repo: https://github.com/tree-sitter/tree-sitter-toml
commit: abc123...
license: MIT

grammars:
  - id: toml
    name: TOML
    tag: config
    tier: 1
    has_scanner: false  # ← No scanner
    generate_component: true
    icon: simple-icons:toml

    inventor: Tom Preston-Werner
    year: 2013
    description: "TOML is a minimal configuration file format designed to be easy to read due to obvious semantics."
    link: https://en.wikipedia.org/wiki/TOML
    trivia: "TOML was created by Tom Preston-Werner, co-founder of GitHub, as a better alternative to INI and YAML for config files."

    samples:
      - path: samples/example.toml
        description: TOML configuration example
        license: MIT
```

**Directory structure** (note: no `scanner.c`):
```
langs/group-maple/toml/def/
├── arborium.yaml
├── grammar/
│   └── grammar.js              (no scanner.c)
├── queries/
│   └── highlights.scm
└── samples/
    └── example.toml
```

---

## Advanced Topics

### Multi-Grammar Crates

Some languages require multiple grammars in one crate. Example: XML + DTD

```yaml
# In group-willow/xml/def/arborium.yaml
grammars:
  - id: xml
    name: XML
    # ... other fields

  - id: dtd
    name: DTD
    grammar_path: dtd  # ← Specify subdirectory
    # ... other fields
```

### WASM Plugin Configuration

Set `generate_component: true` to include in WASM builds:

```yaml
grammars:
  - id: groovy
    generate_component: true  # Build WASM plugin
    # ...
```

**When to enable:**
- ✅ Popular languages (tier 1-2)
- ✅ Languages needed in browser environments
- ❌ Rarely used languages (to reduce bundle size)
- ❌ Very large grammars (>500KB WASM output)

**Check WASM size:**
```bash
cargo xtask build groovy --release
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

### Custom Icon Sources

Icons use [Iconify](https://iconify.design) format: `collection:icon-name`

**Common collections:**
- `devicon-plain:*` - Programming language logos
- `simple-icons:*` - Brand logos
- `vscode-icons:*` - VS Code file type icons
- `file-icons:*` - File type icons

**Example searches:**
- Groovy: `devicon-plain:groovy`
- Java: `devicon-plain:java`
- WebAssembly: `simple-icons:webassembly`
- Generic code: `vscode-icons:file-type-text`

**Fallback:** If no icon exists, use a generic code icon:
```yaml
icon: vscode-icons:file-type-text
```

---

## Checklist

Use this checklist when adding a new language:

### Pre-Implementation
- [ ] Found Tree-sitter grammar repository
- [ ] Verified grammar license is compatible
- [ ] Checked grammar maintenance status (recent commits?)
- [ ] Downloaded sample files for testing
- [ ] Chosen appropriate language group

### Directory Setup
- [ ] Created `langs/group-{name}/{language}/def/` directory
- [ ] Created subdirectories: `grammar/`, `queries/`, `samples/`
- [ ] Copied `grammar.js` from upstream
- [ ] Copied `scanner.c` (if exists)
- [ ] Copied `highlights.scm` from upstream
- [ ] Copied `injections.scm` (if exists)
- [ ] Added at least one sample file

### Configuration
- [ ] Created `arborium.yaml` with all required fields
- [ ] Set correct `has_scanner` value
- [ ] Chose appropriate `tier` (1-5)
- [ ] Found suitable icon from Iconify
- [ ] Added `aliases` (file extensions)
- [ ] Wrote description and trivia
- [ ] Referenced sample files correctly

### Build & Validate
- [ ] Ran `cargo xtask gen {language}`
- [ ] Ran `cargo xtask lint` (no errors)
- [ ] Ran `cargo xtask build {language}` (if WASM enabled)
- [ ] Generated crate files exist in `crate/` directory

### Testing
- [ ] Started dev server: `cargo xtask serve --dev`
- [ ] Opened http://localhost:8080
- [ ] Selected language from dropdown
- [ ] Loaded each sample file
- [ ] Verified syntax highlighting is correct
- [ ] Tested with custom code snippets
- [ ] Checked edge cases (strings, comments, operators)

### Documentation
- [ ] Added entry to `CHANGELOG.md` (if exists)
- [ ] Updated `README.md` with language count (if needed)
- [ ] Considered adding language-specific notes

### Version Control
- [ ] Staged all files: `git add langs/group-{name}/{language}/`
- [ ] Created descriptive commit message
- [ ] Pushed to feature branch
- [ ] Created pull request with description

---

## Getting Help

### Resources

- **Tree-sitter Documentation:** https://tree-sitter.github.io/tree-sitter/
- **Query Syntax:** https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries
- **Iconify Icons:** https://iconify.design
- **SPDX Licenses:** https://spdx.org/licenses/

### Common Questions

**Q: Where can I find Tree-sitter grammars?**

A: Check these sources:
1. Official repository: https://github.com/tree-sitter
2. Community grammars: Search GitHub for "tree-sitter-{language}"
3. Awesome Tree-sitter: https://github.com/tree-sitter/tree-sitter/wiki/List-of-parsers

**Q: What if no Tree-sitter grammar exists?**

A: You have two options:
1. **Write a grammar** (advanced - requires understanding Tree-sitter DSL)
2. **Request one** from the Tree-sitter community

**Q: Can I use a grammar with a different license?**

A: Yes, as long as it's compatible with Arborium's license (check with maintainers). Always document the grammar license in `arborium.yaml`.

**Q: How do I choose between tier 2 and tier 3?**

A:
- **Tier 2:** Grammar is well-maintained, highlighting is accurate, few known issues
- **Tier 3:** Grammar works but has some rough edges, occasional highlighting glitches

When in doubt, start with tier 3 and upgrade after thorough testing.

**Q: What if the grammar doesn't have `highlights.scm`?**

A: You can:
1. Check if the grammar has a `queries/` directory with other files
2. Look for highlights in the grammar's documentation
3. Create a basic one yourself (see Examples section)
4. Copy from a similar language and adapt

**Q: Do I need to understand Rust to add a language?**

A: No! The build system generates all Rust code automatically. You only need to:
- Copy grammar files
- Write `arborium.yaml` configuration
- Run `cargo xtask gen {language}`

---

## Appendix

### File Extension Reference

Common file extensions by group:

| Group | Extensions |
|-------|-----------|
| `group-acorn` | `.js`, `.jsx`, `.ts`, `.tsx`, `.html`, `.css`, `.json` |
| `group-birch` | `.c`, `.h`, `.cpp`, `.cc`, `.rs`, `.zig`, `.asm` |
| `group-cedar` | `.java`, `.kt`, `.scala`, `.clj`, `.groovy` |
| `group-fern` | `.hs`, `.elm`, `.erl`, `.ml`, `.scm` |
| `group-hazel` | `.py`, `.rb`, `.sh`, `.pl`, `.php`, `.lua` |
| `group-maple` | `.sql`, `.yaml`, `.yml`, `.toml`, `.graphql` |
| `group-pine` | `.rs`, `.swift`, `.dart`, `.wit`, `.thrift` |
| `group-sage` | `.cs`, `.fs`, `.vb` |
| `group-willow` | `.md`, `.xml`, `.svg`, `.rst` |

### Tier Guidelines by Language Age

| Age | Typical Tier |
|-----|--------------|
| 20+ years, very popular | Tier 1 (Java, Python, C) |
| 10-20 years, popular | Tier 2 (Kotlin, Go, Rust) |
| 5-10 years, active | Tier 2-3 (Swift, Dart) |
| 0-5 years, experimental | Tier 3-4 (WIT, Pkl) |
| New/incomplete grammar | Tier 4-5 |

### Icon Collection Quick Reference

| Collection | Use For | Examples |
|------------|---------|----------|
| `devicon-plain:*` | Programming languages | `devicon-plain:java`, `devicon-plain:python` |
| `simple-icons:*` | Technology brands | `simple-icons:webassembly`, `simple-icons:kotlin` |
| `vscode-icons:*` | File types | `vscode-icons:file-type-xml` |
| `file-icons:*` | File types | `file-icons:groovy` |

---

## Conclusion

Adding a language to Arborium is straightforward:

1. **Find** a Tree-sitter grammar
2. **Copy** grammar files to `langs/group-{name}/{language}/def/`
3. **Write** `arborium.yaml` configuration
4. **Run** `cargo xtask gen {language}`
5. **Run** `cargo xtask ci generate` (regenerates CI workflow for new language)
6. **Test** in the web demo

The build system handles all code generation automatically. You don't need to understand Rust, Tree-sitter internals, or write complex build scripts.

**Key principles:**
- ✅ `arborium.yaml` is the single source of truth
- ✅ Never edit files in `crate/` (they're auto-generated)
- ✅ Copy grammar files from upstream (don't reinvent)
- ✅ Test thoroughly before submitting

Happy language adding! 🎉
