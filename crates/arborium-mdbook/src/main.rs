use std::borrow::Borrow;
use std::env;
use std::io::{self, Read};
use std::process;

use anyhow::{Context, Result};
use arborium::Highlighter;
use html_escape::{encode_double_quoted_attribute, encode_safe};
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error as MdError;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::{Options as CmarkOptions, cmark_with_options};
use serde::Deserialize;

fn main() -> Result<()> {
    let mut args = env::args();
    let _binary = args.next();

    let preprocess = ArboriumPreprocessor::default();

    if let Some(arg) = args.next() {
        if arg == "supports" {
            let renderer = args.next().unwrap_or_default();
            if preprocess.supports_renderer(&renderer) {
                process::exit(0);
            } else {
                process::exit(1);
            }
        }
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let request: PreprocessorRequest =
        serde_json::from_str(&input).context("invalid mdBook input")?;
    let mut book = request.book;
    preprocess.apply(&request.context, &mut book)?;

    serde_json::to_writer(io::stdout(), &book).context("failed to serialize book")?;

    Ok(())
}

#[derive(Default)]
struct ArboriumPreprocessor;

impl ArboriumPreprocessor {
    fn apply(&self, ctx: &PreprocessorContext, book: &mut Book) -> Result<()> {
        let _ = ctx;
        let mut highlighter = Highlighter::new();

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if chapter.content.trim().is_empty() {
                    return;
                }

                match transform_markdown(&chapter.content, &mut highlighter) {
                    Ok(transformed) => {
                        chapter.content = transformed;
                    }
                    Err(err) => {
                        eprintln!(
                            "[arborium-mdbook] warning: failed to process '{}': {err}",
                            chapter.name
                        );
                    }
                }
            }
        });

        Ok(())
    }
}

impl Preprocessor for ArboriumPreprocessor {
    fn name(&self) -> &str {
        "arborium"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> std::result::Result<Book, MdError> {
        self.apply(ctx, &mut book)
            .map_err(|err| MdError::msg(err.to_string()))?;
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

#[derive(Deserialize)]
struct PreprocessorRequest {
    context: PreprocessorContext,
    book: Book,
}

fn transform_markdown(content: &str, highlighter: &mut Highlighter) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, options);
    let mut events: Vec<Event> = Vec::new();
    let mut active = ActiveFence::default();

    for event in parser {
        let mut handled = false;

        match &event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                active = ActiveFence::Fenced(FencedBlock::new(info.to_string()));
                handled = true;
            }
            Event::Text(text) => {
                if let ActiveFence::Fenced(block) = &mut active {
                    block.push(text.as_ref());
                    handled = true;
                }
            }
            Event::Code(text) => {
                if let ActiveFence::Fenced(block) = &mut active {
                    block.push(text.as_ref());
                    handled = true;
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let ActiveFence::Fenced(block) = &mut active {
                    block.push("\n");
                    handled = true;
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(block) = active.take() {
                    events.push(Event::Html(CowStr::from(block.render(highlighter))));
                    handled = true;
                }
            }
            _ => {}
        }

        if !handled {
            events.push(event);
        }
    }

    if let Some(block) = active.take() {
        events.push(Event::Html(CowStr::from(block.render(highlighter))));
    }

    let mut output = String::new();
    let mut cmark_options = CmarkOptions::default();
    cmark_options.newlines_after_codeblock = 2;
    let borrowable = events.iter().map(EventRef);
    cmark_with_options(borrowable, &mut output, cmark_options)
        .context("failed to serialize Markdown")?;
    Ok(output)
}

#[derive(Default)]
enum ActiveFence {
    #[default]
    Inactive,
    Fenced(FencedBlock),
}

impl ActiveFence {
    fn take(&mut self) -> Option<FencedBlock> {
        match std::mem::replace(self, ActiveFence::Inactive) {
            ActiveFence::Fenced(block) => Some(block),
            ActiveFence::Inactive => None,
        }
    }
}

struct FencedBlock {
    info: String,
    code: String,
}

impl FencedBlock {
    fn new(info: String) -> Self {
        Self {
            info,
            code: String::new(),
        }
    }

    fn push(&mut self, text: &str) {
        self.code.push_str(text);
    }

    fn render(mut self, highlighter: &mut Highlighter) -> String {
        // Trim trailing newline inserted by parser to avoid double spacing.
        if self.code.ends_with('\n') {
            self.code.pop();
            if self.code.ends_with('\r') {
                self.code.pop();
            }
        }

        let lang = parse_language(&self.info);
        let highlighted =
            lang.as_deref()
                .and_then(|lang| match highlighter.highlight(lang, &self.code) {
                    Ok(html) => Some(html),
                    Err(err) => {
                        eprintln!("[arborium-mdbook] unsupported language '{lang}': {err}");
                        None
                    }
                });

        build_code_block_html(
            lang.as_deref(),
            highlighted.unwrap_or_else(|| encode_safe(&self.code).to_string()),
        )
    }
}

fn parse_language(info: &str) -> Option<String> {
    let trimmed = info.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Support CommonMark "lang,option" metadata by splitting on delimiters.
    let token = trimmed
        .trim_start_matches('{')
        .split(|c: char| c == ',' || c.is_whitespace() || c == '}')
        .find(|segment| !segment.is_empty())?;

    Some(token.to_lowercase())
}

fn sanitize_class_token(lang: &str) -> String {
    let mut output = String::with_capacity(lang.len());
    for ch in lang.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '-' | '_') {
            output.push(ch);
        } else {
            output.push('-');
        }
    }

    if output.is_empty() {
        "plain".to_string()
    } else {
        output
    }
}

fn build_code_block_html(language: Option<&str>, body: String) -> String {
    let lang = language.unwrap_or("text");
    let class_token = sanitize_class_token(lang);
    let class_attr = format!("language-{}", class_token);
    let attr_value = encode_double_quoted_attribute(lang);

    format!(
        "\n<pre class=\"{class}\" data-lang=\"{attr}\"><code class=\"{class}\" data-lang=\"{attr}\" tabindex=\"0\">{body}</code></pre>\n",
        class = class_attr,
        attr = attr_value,
        body = body
    )
}

struct EventRef<'a, 'b>(&'b Event<'a>);

impl<'a, 'b> Borrow<pulldown_cmark::Event<'a>> for EventRef<'a, 'b> {
    fn borrow(&self) -> &pulldown_cmark::Event<'a> {
        self.0
    }
}
