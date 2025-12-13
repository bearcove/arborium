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
const languageInfo = <%- language_info %>;

// Examples: maps language id to file extension (e.g. "rust" -> "rs")
// Content fetched on-demand from /samples/{id}.{ext}
const exampleExtensions = <%- examples %>;

// Icons will be injected by generate-demo (SVG strings keyed by iconify name)
const icons = <%- icons %>;

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
const themeInfo = <%- theme_info %>;

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
