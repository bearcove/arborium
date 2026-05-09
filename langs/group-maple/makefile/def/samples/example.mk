# Variables
CC      := clang
CFLAGS  := -O2 -Wall -Wextra
LDFLAGS :=
SRCDIR  := src
OBJDIR  := build
BIN     := $(OBJDIR)/app

SOURCES := $(wildcard $(SRCDIR)/*.c)
OBJECTS := $(SOURCES:$(SRCDIR)/%.c=$(OBJDIR)/%.o)

# Conditional based on environment
ifeq ($(DEBUG),1)
  CFLAGS += -g -O0 -DDEBUG
else
  CFLAGS += -DNDEBUG
endif

UNAME := $(shell uname -s)
ifeq ($(UNAME),Darwin)
  LDFLAGS += -framework CoreFoundation
endif

.PHONY: all clean test install

all: $(BIN)

$(BIN): $(OBJECTS) | $(OBJDIR)
	$(CC) $(LDFLAGS) -o $@ $^

$(OBJDIR)/%.o: $(SRCDIR)/%.c | $(OBJDIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJDIR):
	mkdir -p $@

test: $(BIN)
	./$(BIN) --self-test

install: $(BIN)
	install -m 0755 $(BIN) $(DESTDIR)/usr/local/bin/

clean:
	rm -rf $(OBJDIR)

# Pattern rule
%.tar.gz: %
	tar -czf $@ $<

# Function definition
define greet
	@echo "Hello, $(1)!"
endef

hello:
	$(call greet,world)
