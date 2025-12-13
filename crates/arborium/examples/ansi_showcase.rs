//! ANSI Showcase - Beautiful dark themes with syntax highlighting
//!
//! Run with: cargo run --example ansi_showcase --features showcase

use arborium::AnsiHighlighter;
use arborium::theme::builtin;
use arborium_highlight::AnsiOptions;

fn main() {
    let rust_code = r#"impl<T: Send + Sync> SharedState<T> {
    pub fn new(value: T) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self { value, listeners: vec![] }))
    }

    pub fn update(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.value);
        self.listeners.iter().for_each(|cb| cb(&self.value));
    }
}"#;

    let haskell_code = r#"quicksort :: Ord a => [a] -> [a]
quicksort [] = []
quicksort (x:xs) = quicksort smaller ++ [x] ++ quicksort larger
  where
    smaller = [a | a <- xs, a <= x]
    larger  = [b | b <- xs, b > x]

main :: IO ()
main = print $ quicksort [3, 1, 4, 1, 5, 9, 2, 6]"#;

    let svelte_code = r#"<script lang="ts">
  let count = $state(0);
  let doubled = $derived(count * 2);

  function increment() {
    count += 1;
  }
</script>

<button onclick={increment}>
  Clicked {count} times (doubled: {doubled})
</button>"#;

    let scss_code = r#"$primary: #6366f1;
$radius: 0.5rem;

@mixin glass-effect($blur: 12px) {
  backdrop-filter: blur($blur);
  background: rgba(white, 0.1);
  border: 1px solid rgba(white, 0.2);
}

.card {
  @include glass-effect;
  border-radius: $radius;
  padding: 1.5rem;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 32px rgba($primary, 0.3);
  }
}"#;

    let showcase: &[(&str, fn() -> &'static arborium_theme::Theme, &str, &str)] = &[
        ("Tokyo Night", builtin::tokyo_night, "rust", rust_code),
        (
            "Kanagawa Dragon",
            builtin::kanagawa_dragon,
            "haskell",
            haskell_code,
        ),
        (
            "Rose Pine Moon",
            builtin::rose_pine_moon,
            "svelte",
            svelte_code,
        ),
        (
            "Catppuccin Mocha",
            builtin::catppuccin_mocha,
            "scss",
            scss_code,
        ),
    ];

    println!("\n{}", "═".repeat(70));
    println!("  ARBORIUM SHOWCASE - Dark Themes");
    println!("{}\n", "═".repeat(70));

    for (theme_name, theme_fn, lang, code) in showcase {
        println!("  {theme_name} — {lang}\n");

        let theme = theme_fn().clone();
        let config = arborium::Config::default();
        let mut options = AnsiOptions::default();
        options.use_theme_base_style = true;
        options.width = Some(60);
        options.pad_to_width = true;
        options.padding_x = 2;
        options.padding_y = 1;
        options.border = true;

        let mut hl = AnsiHighlighter::with_options(theme, config, options);
        let output = hl.highlight(lang, code).unwrap();
        println!("{output}\n");
    }

    println!("{}", "═".repeat(70));
}
