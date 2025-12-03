const KEYWORDS = {
  FUNCTION: {
    mandat: "fu",
    opt: "nction",
    ignore_comments_after: false,
  },
  ENDFUNCTION: {
    mandat: "endf",
    opt: "unction",
    ignore_comments_after: false,
  },
  FOR: {
    mandat: "for",
    opt: "",
    ignore_comments_after: false,
  },
  ENDFOR: {
    mandat: "endfo",
    opt: "r",
    ignore_comments_after: false,
  },
  WHILE: {
    mandat: "wh",
    opt: "ile",
    ignore_comments_after: false,
  },
  ENDWHILE: {
    mandat: "endw",
    opt: "hile",
    ignore_comments_after: false,
  },
  IF: {
    mandat: "if",
    opt: "",
    ignore_comments_after: false,
  },
  ELSEIF: {
    mandat: "elsei",
    opt: "f",
    ignore_comments_after: false,
  },
  ELSE: {
    mandat: "el",
    opt: "se",
    ignore_comments_after: false,
  },
  ENDIF: {
    mandat: "en",
    opt: "dif",
    ignore_comments_after: false,
  },
  TRY: {
    mandat: "try",
    opt: "",
    ignore_comments_after: false,
  },
  CATCH: {
    mandat: "cat",
    opt: "ch",
    ignore_comments_after: false,
  },
  CNEXT: {
    mandat: "cn",
    opt: "ext",
    ignore_comments_after: false,
  },
  CPREVIOUS: {
    mandat: "cp",
    opt: "revious",
    ignore_comments_after: false,
  },
  CNNEXT: {
    mandat: "cN",
    opt: "ext",
    ignore_comments_after: false,
  },
  FINALLY: {
    mandat: "fina",
    opt: "lly",
    ignore_comments_after: false,
  },
  ENDTRY: {
    mandat: "endt",
    opt: "ry",
    ignore_comments_after: false,
  },
  CONST: {
    mandat: "cons",
    opt: "t",
    ignore_comments_after: false,
  },
  NORMAL: {
    mandat: "norm",
    opt: "al",
    ignore_comments_after: false,
  },
  RETURN: {
    mandat: "retu",
    opt: "rn",
    ignore_comments_after: false,
  },
  PERL: {
    mandat: "perl",
    opt: "",
    ignore_comments_after: false,
  },
  LUA: {
    mandat: "lua",
    opt: "",
    ignore_comments_after: false,
  },
  RUBY: {
    mandat: "rub",
    opt: "y",
    ignore_comments_after: false,
  },
  PYTHON: {
    mandat: "py",
    opt: "thon",
    ignore_comments_after: false,
  },
  THROW: {
    mandat: "th",
    opt: "row",
    ignore_comments_after: false,
  },
  EXECUTE: {
    mandat: "exe",
    opt: "cute",
    ignore_comments_after: false,
  },
  AUTOCMD: {
    mandat: "au",
    opt: "tocmd",
    ignore_comments_after: false,
  },
  SILENT: {
    mandat: "sil",
    opt: "ent",
    ignore_comments_after: false,
  },
  ECHO: {
    mandat: "ec",
    opt: "ho",
    ignore_comments_after: true,
  },
  ECHON: {
    mandat: "echon",
    opt: "",
    ignore_comments_after: true,
  },
  ECHOHL: {
    mandat: "echoh",
    opt: "l",
    ignore_comments_after: false,
  },
  ECHOMSG: {
    mandat: "echom",
    opt: "sg",
    ignore_comments_after: true,
  },
  ECHOERR: {
    mandat: "echoe",
    opt: "rr",
    ignore_comments_after: true,
  },
  MAP: {
    mandat: "map",
    opt: "",
    ignore_comments_after: true,
  },
  NMAP: {
    mandat: "nm",
    opt: "ap",
    ignore_comments_after: true,
  },
  VMAP: {
    mandat: "vm",
    opt: "ap",
    ignore_comments_after: true,
  },
  XMAP: {
    mandat: "xm",
    opt: "ap",
    ignore_comments_after: true,
  },
  SMAP: {
    mandat: "smap",
    opt: "",
    ignore_comments_after: true,
  },
  OMAP: {
    mandat: "om",
    opt: "ap",
    ignore_comments_after: true,
  },
  IMAP: {
    mandat: "im",
    opt: "ap",
    ignore_comments_after: true,
  },
  LMAP: {
    mandat: "lm",
    opt: "ap",
    ignore_comments_after: true,
  },
  CMAP: {
    mandat: "cm",
    opt: "ap",
    ignore_comments_after: true,
  },
  TMAP: {
    mandat: "tma",
    opt: "p",
    ignore_comments_after: true,
  },
  NOREMAP: {
    mandat: "no",
    opt: "remap",
    ignore_comments_after: true,
  },
  NNOREMAP: {
    mandat: "nn",
    opt: "oremap",
    ignore_comments_after: true,
  },
  VNOREMAP: {
    mandat: "vn",
    opt: "oremap",
    ignore_comments_after: true,
  },
  XNOREMAP: {
    mandat: "xn",
    opt: "oremap",
    ignore_comments_after: true,
  },
  SNOREMAP: {
    mandat: "snor",
    opt: "emap",
    ignore_comments_after: true,
  },
  ONOREMAP: {
    mandat: "ono",
    opt: "remap",
    ignore_comments_after: true,
  },
  INOREMAP: {
    mandat: "ino",
    opt: "remap",
    ignore_comments_after: true,
  },
  LNOREMAP: {
    mandat: "ln",
    opt: "oremap",
    ignore_comments_after: true,
  },
  CNOREMAP: {
    mandat: "cno",
    opt: "remap",
    ignore_comments_after: true,
  },
  TNOREMAP: {
    mandat: "tno",
    opt: "remap",
    ignore_comments_after: true,
  },
  AUGROUP: {
    mandat: "aug",
    opt: "roup",
    ignore_comments_after: true,
  },
  HIGHLIGHT: {
    mandat: "hi",
    opt: "ghlight",
    ignore_comments_after: false,
  },
  DEFAULT: {
    mandat: "def",
    opt: "ault",
    ignore_comments_after: false,
  }, // highlight def[ault },
  SYNTAX: {
    mandat: "sy",
    opt: "ntax",
    ignore_comments_after: false,
  },
  SET: {
    mandat: "se",
    opt: "t",
    ignore_comments_after: false,
  },
  SETLOCAL: {
    mandat: "setl",
    opt: "ocal",
    ignore_comments_after: false,
  },
  SETFILETYPE: {
    mandat: "setf",
    opt: "iletype",
    ignore_comments_after: false,
  },
  BROWSE: {
    mandat: "bro",
    opt: "wse",
    ignore_comments_after: false,
  },
  OPTIONS: {
    mandat: "opt",
    opt: "ions",
    ignore_comments_after: false,
  },
  STARTINSERT: {
    mandat: "star",
    opt: "tinsert",
    ignore_comments_after: false,
  },
  STOPINSERT: {
    mandat: "stopi",
    opt: "nsert",
    ignore_comments_after: false,
  },
  SCRIPTENCODING: {
    mandat: "scripte",
    opt: "ncoding",
    ignore_comments_after: false,
  },
  SOURCE: {
    mandat: "so",
    opt: "urce",
    ignore_comments_after: false,
  },
  GLOBAL: {
    mandat: "g",
    opt: "lobal",
    ignore_comments_after: false,
  },
  COLORSCHEME: {
    mandat: "colo",
    opt: "rscheme",
    ignore_comments_after: false,
  },
  COMMAND: {
    mandat: "com",
    opt: "mand",
    ignore_comments_after: false,
  },
  COMCLEAR: {
    mandat: "comc",
    opt: "lear",
    ignore_comments_after: false,
  },
  DELCOMMAND: {
    mandat: "delc",
    opt: "ommand",
    ignore_comments_after: false,
  },
  RUNTIME: {
    mandat: "ru",
    opt: "ntime",
    ignore_comments_after: false,
  },
  WINCMD: {
    mandat: "winc",
    opt: "md",
    ignore_comments_after: false,
  },
  SIGN: {
    mandat: "sig",
    opt: "n",
    ignore_comments_after: false,
  },
  FILETYPE: {
    mandat: "filet",
    opt: "ype",
    ignore_comments_after: false,
  },
  LET: {
    mandat: "let",
    opt: "",
    ignore_comments_after: false,
  },
  UNLET: {
    mandat: "unl",
    opt: "et",
    ignore_comments_after: false,
  },
  CALL: {
    mandat: "cal",
    opt: "l",
    ignore_comments_after: false,
  },
  BREAK: {
    mandat: "brea",
    opt: "k",
    ignore_comments_after: false,
  },
  CONTINUE: {
    mandat: "con",
    opt: "tinue",
    ignore_comments_after: false,
  },
  VERTICAL: {
    mandat: "vert",
    opt: "ical",
    ignore_comments_after: false,
  },
  LEFTABOVE: {
    mandat: "lefta",
    opt: "bove",
    ignore_comments_after: false,
  },
  ABOVELEFT: {
    mandat: "abo",
    opt: "veleft",
    ignore_comments_after: false,
  },
  RIGHTBELOWS: {
    mandat: "rightb",
    opt: "elow",
    ignore_comments_after: false,
  },
  BELOWRIGHT: {
    mandat: "bel",
    opt: "owright",
    ignore_comments_after: false,
  },
  TOPLEFT: {
    mandat: "to",
    opt: "pleft",
    ignore_comments_after: false,
  },
  BOTRIGHT: {
    mandat: "bo",
    opt: "tright",
    ignore_comments_after: false,
  },
  EDIT: {
    mandat: "e",
    opt: "dit",
    ignore_comments_after: false,
  },
  ENEW: {
    mandat: "ene",
    opt: "w",
    ignore_comments_after: false,
  },
  FIND: {
    mandat: "fin",
    opt: "d",
    ignore_comments_after: false,
  },
  EX: {
    mandat: "ex",
    opt: "",
    ignore_comments_after: false,
  },
  VISUAL: {
    mandat: "vi",
    opt: "sual",
    ignore_comments_after: false,
  },
  VIEW: {
    mandat: "vie",
    opt: "w",
    ignore_comments_after: false,
  },
  EVAL: {
    mandat: "ev",
    opt: "al",
    ignore_comments_after: false,
  },
  SUBSTITUTE: {
    mandat: "s",
    opt: "ubstitute",
    ignore_comments_after: false,
  },
};

function make_keywords($) {
  const fs = require("fs");
  const path = require("path");
  const KEYWORDS_FILE = path.join("src", "keywords.h");

  let rules = [];

  fs.writeFileSync(
    KEYWORDS_FILE,
    `typedef enum {
`,
    (err) => {}
  );

  for (const [kname, infos] of Object.entries(KEYWORDS)) {
    fs.appendFileSync(
      KEYWORDS_FILE,
      `  ${kname} = ${rules.length},\n`,
      (err) => {}
    );
    rules.push($["_" + infos.mandat + infos.opt]);
  }

  fs.appendFileSync(
    KEYWORDS_FILE,
    `  UNKNOWN_COMMAND
} kwid;

keyword keywords[] = {
`,
    (err) => {}
  );
  rules.push($.unknown_command_name);

  for (const [kname, infos] of Object.entries(KEYWORDS)) {
    fs.appendFileSync(
      KEYWORDS_FILE,
      `  [${kname}] = {
    .mandat = "${infos.mandat}",
    .opt = "${infos.opt}",
    .ignore_comments_after = ${infos.ignore_comments_after}
  },\n`,
      (err) => {}
    );
  }

  fs.appendFileSync(KEYWORDS_FILE, "};", (err) => {});

  return rules;
}

module.exports = {
  keywords: make_keywords,
};
