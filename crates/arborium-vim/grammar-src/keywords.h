typedef enum {
  FUNCTION = 0,
  ENDFUNCTION = 1,
  FOR = 2,
  ENDFOR = 3,
  WHILE = 4,
  ENDWHILE = 5,
  IF = 6,
  ELSEIF = 7,
  ELSE = 8,
  ENDIF = 9,
  TRY = 10,
  CATCH = 11,
  CNEXT = 12,
  CPREVIOUS = 13,
  CNNEXT = 14,
  FINALLY = 15,
  ENDTRY = 16,
  CONST = 17,
  NORMAL = 18,
  RETURN = 19,
  PERL = 20,
  LUA = 21,
  RUBY = 22,
  PYTHON = 23,
  THROW = 24,
  EXECUTE = 25,
  AUTOCMD = 26,
  SILENT = 27,
  ECHO = 28,
  ECHON = 29,
  ECHOHL = 30,
  ECHOMSG = 31,
  ECHOERR = 32,
  MAP = 33,
  NMAP = 34,
  VMAP = 35,
  XMAP = 36,
  SMAP = 37,
  OMAP = 38,
  IMAP = 39,
  LMAP = 40,
  CMAP = 41,
  TMAP = 42,
  NOREMAP = 43,
  NNOREMAP = 44,
  VNOREMAP = 45,
  XNOREMAP = 46,
  SNOREMAP = 47,
  ONOREMAP = 48,
  INOREMAP = 49,
  LNOREMAP = 50,
  CNOREMAP = 51,
  TNOREMAP = 52,
  AUGROUP = 53,
  HIGHLIGHT = 54,
  DEFAULT = 55,
  SYNTAX = 56,
  SET = 57,
  SETLOCAL = 58,
  SETFILETYPE = 59,
  BROWSE = 60,
  OPTIONS = 61,
  STARTINSERT = 62,
  STOPINSERT = 63,
  SCRIPTENCODING = 64,
  SOURCE = 65,
  GLOBAL = 66,
  COLORSCHEME = 67,
  COMMAND = 68,
  COMCLEAR = 69,
  DELCOMMAND = 70,
  RUNTIME = 71,
  WINCMD = 72,
  SIGN = 73,
  FILETYPE = 74,
  LET = 75,
  UNLET = 76,
  CALL = 77,
  BREAK = 78,
  CONTINUE = 79,
  VERTICAL = 80,
  LEFTABOVE = 81,
  ABOVELEFT = 82,
  RIGHTBELOWS = 83,
  BELOWRIGHT = 84,
  TOPLEFT = 85,
  BOTRIGHT = 86,
  EDIT = 87,
  ENEW = 88,
  FIND = 89,
  EX = 90,
  VISUAL = 91,
  VIEW = 92,
  EVAL = 93,
  SUBSTITUTE = 94,
  UNKNOWN_COMMAND
} kwid;

keyword keywords[] = {
  [FUNCTION] = {
    .mandat = "fu",
    .opt = "nction",
    .ignore_comments_after = false
  },
  [ENDFUNCTION] = {
    .mandat = "endf",
    .opt = "unction",
    .ignore_comments_after = false
  },
  [FOR] = {
    .mandat = "for",
    .opt = "",
    .ignore_comments_after = false
  },
  [ENDFOR] = {
    .mandat = "endfo",
    .opt = "r",
    .ignore_comments_after = false
  },
  [WHILE] = {
    .mandat = "wh",
    .opt = "ile",
    .ignore_comments_after = false
  },
  [ENDWHILE] = {
    .mandat = "endw",
    .opt = "hile",
    .ignore_comments_after = false
  },
  [IF] = {
    .mandat = "if",
    .opt = "",
    .ignore_comments_after = false
  },
  [ELSEIF] = {
    .mandat = "elsei",
    .opt = "f",
    .ignore_comments_after = false
  },
  [ELSE] = {
    .mandat = "el",
    .opt = "se",
    .ignore_comments_after = false
  },
  [ENDIF] = {
    .mandat = "en",
    .opt = "dif",
    .ignore_comments_after = false
  },
  [TRY] = {
    .mandat = "try",
    .opt = "",
    .ignore_comments_after = false
  },
  [CATCH] = {
    .mandat = "cat",
    .opt = "ch",
    .ignore_comments_after = false
  },
  [CNEXT] = {
    .mandat = "cn",
    .opt = "ext",
    .ignore_comments_after = false
  },
  [CPREVIOUS] = {
    .mandat = "cp",
    .opt = "revious",
    .ignore_comments_after = false
  },
  [CNNEXT] = {
    .mandat = "cN",
    .opt = "ext",
    .ignore_comments_after = false
  },
  [FINALLY] = {
    .mandat = "fina",
    .opt = "lly",
    .ignore_comments_after = false
  },
  [ENDTRY] = {
    .mandat = "endt",
    .opt = "ry",
    .ignore_comments_after = false
  },
  [CONST] = {
    .mandat = "cons",
    .opt = "t",
    .ignore_comments_after = false
  },
  [NORMAL] = {
    .mandat = "norm",
    .opt = "al",
    .ignore_comments_after = false
  },
  [RETURN] = {
    .mandat = "retu",
    .opt = "rn",
    .ignore_comments_after = false
  },
  [PERL] = {
    .mandat = "perl",
    .opt = "",
    .ignore_comments_after = false
  },
  [LUA] = {
    .mandat = "lua",
    .opt = "",
    .ignore_comments_after = false
  },
  [RUBY] = {
    .mandat = "rub",
    .opt = "y",
    .ignore_comments_after = false
  },
  [PYTHON] = {
    .mandat = "py",
    .opt = "thon",
    .ignore_comments_after = false
  },
  [THROW] = {
    .mandat = "th",
    .opt = "row",
    .ignore_comments_after = false
  },
  [EXECUTE] = {
    .mandat = "exe",
    .opt = "cute",
    .ignore_comments_after = false
  },
  [AUTOCMD] = {
    .mandat = "au",
    .opt = "tocmd",
    .ignore_comments_after = false
  },
  [SILENT] = {
    .mandat = "sil",
    .opt = "ent",
    .ignore_comments_after = false
  },
  [ECHO] = {
    .mandat = "ec",
    .opt = "ho",
    .ignore_comments_after = true
  },
  [ECHON] = {
    .mandat = "echon",
    .opt = "",
    .ignore_comments_after = true
  },
  [ECHOHL] = {
    .mandat = "echoh",
    .opt = "l",
    .ignore_comments_after = false
  },
  [ECHOMSG] = {
    .mandat = "echom",
    .opt = "sg",
    .ignore_comments_after = true
  },
  [ECHOERR] = {
    .mandat = "echoe",
    .opt = "rr",
    .ignore_comments_after = true
  },
  [MAP] = {
    .mandat = "map",
    .opt = "",
    .ignore_comments_after = true
  },
  [NMAP] = {
    .mandat = "nm",
    .opt = "ap",
    .ignore_comments_after = true
  },
  [VMAP] = {
    .mandat = "vm",
    .opt = "ap",
    .ignore_comments_after = true
  },
  [XMAP] = {
    .mandat = "xm",
    .opt = "ap",
    .ignore_comments_after = true
  },
  [SMAP] = {
    .mandat = "smap",
    .opt = "",
    .ignore_comments_after = true
  },
  [OMAP] = {
    .mandat = "om",
    .opt = "ap",
    .ignore_comments_after = true
  },
  [IMAP] = {
    .mandat = "im",
    .opt = "ap",
    .ignore_comments_after = true
  },
  [LMAP] = {
    .mandat = "lm",
    .opt = "ap",
    .ignore_comments_after = true
  },
  [CMAP] = {
    .mandat = "cm",
    .opt = "ap",
    .ignore_comments_after = true
  },
  [TMAP] = {
    .mandat = "tma",
    .opt = "p",
    .ignore_comments_after = true
  },
  [NOREMAP] = {
    .mandat = "no",
    .opt = "remap",
    .ignore_comments_after = true
  },
  [NNOREMAP] = {
    .mandat = "nn",
    .opt = "oremap",
    .ignore_comments_after = true
  },
  [VNOREMAP] = {
    .mandat = "vn",
    .opt = "oremap",
    .ignore_comments_after = true
  },
  [XNOREMAP] = {
    .mandat = "xn",
    .opt = "oremap",
    .ignore_comments_after = true
  },
  [SNOREMAP] = {
    .mandat = "snor",
    .opt = "emap",
    .ignore_comments_after = true
  },
  [ONOREMAP] = {
    .mandat = "ono",
    .opt = "remap",
    .ignore_comments_after = true
  },
  [INOREMAP] = {
    .mandat = "ino",
    .opt = "remap",
    .ignore_comments_after = true
  },
  [LNOREMAP] = {
    .mandat = "ln",
    .opt = "oremap",
    .ignore_comments_after = true
  },
  [CNOREMAP] = {
    .mandat = "cno",
    .opt = "remap",
    .ignore_comments_after = true
  },
  [TNOREMAP] = {
    .mandat = "tno",
    .opt = "remap",
    .ignore_comments_after = true
  },
  [AUGROUP] = {
    .mandat = "aug",
    .opt = "roup",
    .ignore_comments_after = true
  },
  [HIGHLIGHT] = {
    .mandat = "hi",
    .opt = "ghlight",
    .ignore_comments_after = false
  },
  [DEFAULT] = {
    .mandat = "def",
    .opt = "ault",
    .ignore_comments_after = false
  },
  [SYNTAX] = {
    .mandat = "sy",
    .opt = "ntax",
    .ignore_comments_after = false
  },
  [SET] = {
    .mandat = "se",
    .opt = "t",
    .ignore_comments_after = false
  },
  [SETLOCAL] = {
    .mandat = "setl",
    .opt = "ocal",
    .ignore_comments_after = false
  },
  [SETFILETYPE] = {
    .mandat = "setf",
    .opt = "iletype",
    .ignore_comments_after = false
  },
  [BROWSE] = {
    .mandat = "bro",
    .opt = "wse",
    .ignore_comments_after = false
  },
  [OPTIONS] = {
    .mandat = "opt",
    .opt = "ions",
    .ignore_comments_after = false
  },
  [STARTINSERT] = {
    .mandat = "star",
    .opt = "tinsert",
    .ignore_comments_after = false
  },
  [STOPINSERT] = {
    .mandat = "stopi",
    .opt = "nsert",
    .ignore_comments_after = false
  },
  [SCRIPTENCODING] = {
    .mandat = "scripte",
    .opt = "ncoding",
    .ignore_comments_after = false
  },
  [SOURCE] = {
    .mandat = "so",
    .opt = "urce",
    .ignore_comments_after = false
  },
  [GLOBAL] = {
    .mandat = "g",
    .opt = "lobal",
    .ignore_comments_after = false
  },
  [COLORSCHEME] = {
    .mandat = "colo",
    .opt = "rscheme",
    .ignore_comments_after = false
  },
  [COMMAND] = {
    .mandat = "com",
    .opt = "mand",
    .ignore_comments_after = false
  },
  [COMCLEAR] = {
    .mandat = "comc",
    .opt = "lear",
    .ignore_comments_after = false
  },
  [DELCOMMAND] = {
    .mandat = "delc",
    .opt = "ommand",
    .ignore_comments_after = false
  },
  [RUNTIME] = {
    .mandat = "ru",
    .opt = "ntime",
    .ignore_comments_after = false
  },
  [WINCMD] = {
    .mandat = "winc",
    .opt = "md",
    .ignore_comments_after = false
  },
  [SIGN] = {
    .mandat = "sig",
    .opt = "n",
    .ignore_comments_after = false
  },
  [FILETYPE] = {
    .mandat = "filet",
    .opt = "ype",
    .ignore_comments_after = false
  },
  [LET] = {
    .mandat = "let",
    .opt = "",
    .ignore_comments_after = false
  },
  [UNLET] = {
    .mandat = "unl",
    .opt = "et",
    .ignore_comments_after = false
  },
  [CALL] = {
    .mandat = "cal",
    .opt = "l",
    .ignore_comments_after = false
  },
  [BREAK] = {
    .mandat = "brea",
    .opt = "k",
    .ignore_comments_after = false
  },
  [CONTINUE] = {
    .mandat = "con",
    .opt = "tinue",
    .ignore_comments_after = false
  },
  [VERTICAL] = {
    .mandat = "vert",
    .opt = "ical",
    .ignore_comments_after = false
  },
  [LEFTABOVE] = {
    .mandat = "lefta",
    .opt = "bove",
    .ignore_comments_after = false
  },
  [ABOVELEFT] = {
    .mandat = "abo",
    .opt = "veleft",
    .ignore_comments_after = false
  },
  [RIGHTBELOWS] = {
    .mandat = "rightb",
    .opt = "elow",
    .ignore_comments_after = false
  },
  [BELOWRIGHT] = {
    .mandat = "bel",
    .opt = "owright",
    .ignore_comments_after = false
  },
  [TOPLEFT] = {
    .mandat = "to",
    .opt = "pleft",
    .ignore_comments_after = false
  },
  [BOTRIGHT] = {
    .mandat = "bo",
    .opt = "tright",
    .ignore_comments_after = false
  },
  [EDIT] = {
    .mandat = "e",
    .opt = "dit",
    .ignore_comments_after = false
  },
  [ENEW] = {
    .mandat = "ene",
    .opt = "w",
    .ignore_comments_after = false
  },
  [FIND] = {
    .mandat = "fin",
    .opt = "d",
    .ignore_comments_after = false
  },
  [EX] = {
    .mandat = "ex",
    .opt = "",
    .ignore_comments_after = false
  },
  [VISUAL] = {
    .mandat = "vi",
    .opt = "sual",
    .ignore_comments_after = false
  },
  [VIEW] = {
    .mandat = "vie",
    .opt = "w",
    .ignore_comments_after = false
  },
  [EVAL] = {
    .mandat = "ev",
    .opt = "al",
    .ignore_comments_after = false
  },
  [SUBSTITUTE] = {
    .mandat = "s",
    .opt = "ubstitute",
    .ignore_comments_after = false
  },
};