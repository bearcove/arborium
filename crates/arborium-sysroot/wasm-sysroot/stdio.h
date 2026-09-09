#ifndef _STDIO_H
#define _STDIO_H

#define NULL ((void*)0)

typedef unsigned long size_t;
typedef struct {} FILE;

// Stdio macros discard their arguments, so `stderr` must not require a symbol.
#define stderr ((FILE *)0)

#define fprintf(stream, ...) 0
#define snprintf(str, size, format, ...) 0
#define vsnprintf(str, size, format, ap) 0
#define fputs(s, stream) 0
#define fputc(c, stream) 0
#define fdopen(fd, mode) NULL
#define fclose(stream) 0

#endif
