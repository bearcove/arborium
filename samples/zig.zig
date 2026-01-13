// SPDX-License-Identifier: MIT

const Arg = @import("arg.zig").Arg;

pub fn verifyIter(comptime Iter: type) void {
    if (@hasDecl(Iter, "next") == false) {
        @compileLog(Iter);
        @compileError("The Iter type must have a `pub fn next` method.");
    }

    switch (@TypeOf(Iter.next)) {
        fn (*Iter) ?[:0]const u8,
        fn (*Iter) ?[:0]u8,
        => {},
        else => {
            @compileLog(Iter, Iter.next);
            @compileError("The Iter.next method must be `pub fn next(*Self) ?[:0] const u8`.");
        },
    }

    if (@hasDecl(Iter, "deinit")) {
        switch (@TypeOf(Iter.deinit)) {
            fn (*Iter) void,
            fn (*const Iter) void,
            => {},
            else => {
                @compileLog(Iter, Iter.deinit);
                @compileError("The Iter.deinit method must be `pub fn deinit(Self) void`.");
            },
        }
    }

    if (@hasDecl(Iter, "reset")) {
        switch (@TypeOf(Iter.reset)) {
            fn (*Iter) void,
            fn (*const Iter) void,
            => {},
            else => {
                @compileLog(Iter, Iter.reset);
                @compileError("The Iter.reset method must be `pub fn reset(Self) void`.");
            },
        }
    }
}

/// A generic lexer over command line arguments.
/// The `Iter` type returns the next command line argument.
/// Any quoting, e.g. turning `"foo bar"` into a single argument must be done
/// by the Iter implementation.
pub fn GenericArgs(comptime Iter: type, comptime handle_escape: bool) type {
    comptime verifyIter(Iter);

    return struct {
        iter: Iter,
        peeked: ?RawArg = null,
        has_seen_escape: if (handle_escape) bool else void = if (handle_escape) false else {},

        const Self = @This();
        pub const Inner = Iter;

        /// Create a new Args lexer using the provided iterator.
        /// This can be used to control exactly how the arguments are read.
        /// The iterator must have a function:
        ///
        /// ```
        /// pub fn next(self: *Self) ?[:0]const u8 {}
        /// ```
        pub fn init(iter: Iter) Self {
            return .{ .iter = iter };
        }

        /// Free the internal iterator's buffers.
        /// Invalidates all slices in any arg returned from this Args.
        pub fn deinit(self: *Self) void {
            self.peeked = null;
            if (@hasDecl(Iter, "deinit")) {
                self.iter.deinit();
            }
        }

        /// Checks if calling `reset` is supported.
        pub fn canReset(self: *const Self) bool {
            _ = self;
            return @hasDecl(Iter, "reset");
        }

        /// Reset the underlying iterator, if it is supported.
        pub fn reset(self: *Self) void {
            if (!@hasDecl(Iter, "reset")) {
                @compileError(@typeName(Iter) ++ " does not support reset.");
            }

            self.iter.reset();
        }

        /// Peek at the next `Arg` without consuming it.
        /// Repeated calls will return the same `Arg`.
        pub fn peek(self: *Self) ?Arg {
            if (self.peeked == null) {
                const next_raw = self.iter.next() orelse return null;
                const next_arg: Arg = if (comptime handle_escape) arg: {
                    if (self.has_seen_escape) {
                        break :arg .{ .value = next_raw };
                    } else {
                        const next_arg = parseArg(next_raw);
                        if (next_arg == .escape) {
                            self.has_seen_escape = true;
                            return self.peek();
                        }
                        break :arg next_arg;
                    }
                } else parseArg(next_raw);

                self.peeked = .{ .arg = next_arg, .raw = next_raw };
            }
            return self.peeked.?.arg;
        }

        /// Peek at the next argument as a value, even if it looks like a flag.
        pub fn peekAsValue(self: *Self) ?[:0]const u8 {
            _ = self.peek() orelse return null;
            return self.peeked.?.raw;
        }

        /// Return the next `Arg`.
        pub fn next(self: *Self) ?Arg {
            _ = self.peek() orelse return null;
            defer self.peeked = null;
            return self.peeked.?.arg;
        }

        /// Return the next arg as a value, even if it looks like a flag.
        pub fn nextAsValue(self: *Self) ?[:0]const u8 {
            _ = self.peek() orelse return null;
            defer self.peeked = null;
            return self.peeked.?.raw;
        }

        /// Returns the next args only if it is a value.
        pub fn nextIfValue(self: *Self) ?[:0]const u8 {
            const peeked = self.peek() orelse return null;
            if (peeked == .value) {
                defer self.peeked = null;
                return peeked.value;
            } else {
                return null;
            }
        }

        /// Skips to next argument without parsing it.
        /// Returns true if an argument was skipped,
        /// false if there are no more arguments.
        pub fn skip(self: *Self) bool {
            return self.nextAsValue() != null;
        }

        fn parseArg(arg: [:0]const u8) Arg {
            if (std.mem.startsWith(u8, arg, "--")) {
                const long_arg = arg[2..];
                if (long_arg.len == 0) {
                    return .{ .escape = {} };
                }

                if (std.mem.indexOfScalar(u8, long_arg, '=')) |eql_index| {
                    const flag = long_arg[0..eql_index];
                    const value = long_arg[eql_index + 1 ..];

                    return .{ .long = .{ .flag = flag, .value = value } };
                } else {
                    return .{ .long = .{ .flag = long_arg } };
                }
            }

            if (std.mem.startsWith(u8, arg, "-") and arg.len > 1) {
                const short_args = arg[1..];
                return .{ .shorts = .{ .flags = short_args } };
            }

            return .{ .value = arg };
        }

        /// Reconfigure this args lexer to handle the `--` escape sequence.
        pub fn handleEscape(self: Self) GenericArgs(Iter, true) {
            if (comptime handle_escape) return self;
            return .{ .iter = self.iter, .peeked = self.peeked, .has_seen_escape = false };
        }

        /// Reconfigure this args lexer to yield the `--` escape sequence as `.escape`.
        pub fn yieldEscape(self: Self) GenericArgs(Iter, false) {
            if (!handle_escape) return self;
            return .{ .iter = self.iter, .peeked = self.peeked };
        }

        const RawArg = struct {
            arg: Arg,
            raw: [:0]const u8,
        };
    };
}

const t = std.testing;
const mkArgs = @import("args.zig").SliceArgs.init;

test "stdio" {
    var args = mkArgs(&.{ "bin", "-" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualStrings("-", args.next().?.value);
    try t.expect(args.next() == null);
}

test "handles escape" {
    var args = mkArgs(&.{ "bin", "--" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expect(args.next() == null);
}

test "yields escape" {
    var args = mkArgs(&.{ "bin", "--" }).yieldEscape();
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(.escape, args.next().?);
    try t.expect(args.next() == null);
}

test "long with no value" {
    var args = mkArgs(&.{ "bin", "--long" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "long" }, args.next().?.long);
    try t.expect(args.next() == null);
}

test "long with value" {
    var args = mkArgs(&.{ "bin", "--long=value" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "long", .value = "value" }, args.next().?.long);
    try t.expect(args.next() == null);
}

test "long with value and equals" {
    var args = mkArgs(&.{ "bin", "--long=value=with=equals" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "long", .value = "value=with=equals" }, args.next().?.long);
    try t.expect(args.next() == null);
}

test "long with empty value" {
    var args = mkArgs(&.{ "bin", "--long=" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "long", .value = "" }, args.next().?.long);
    try t.expect(args.next() == null);
}

test "long with space separated value" {
    var args = mkArgs(&.{ "bin", "--long", "space" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "long" }, args.next().?.long);
    try t.expectEqualStrings("space", args.nextAsValue().?);
    try t.expect(args.next() == null);
}

test "single short" {
    var args = mkArgs(&.{ "bin", "-a" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(Arg.Shorts{ .flags = "a" }, args.next().?.shorts);
    try t.expect(args.next() == null);
}

test "combined shorts" {
    var args = mkArgs(&.{ "bin", "-abc" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(Arg.Shorts{ .flags = "abc" }, args.next().?.shorts);
    try t.expect(args.next() == null);
}

test "value" {
    var args = mkArgs(&.{ "bin", "value" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualStrings("value", args.next().?.value);
    try t.expect(args.next() == null);
}

test "combined long, shorts, and value" {
    var args = mkArgs(&.{ "bin", "--long=value", "-abc", "value" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "long", .value = "value" }, args.next().?.long);
    try t.expectEqualDeep(Arg.Shorts{ .flags = "abc" }, args.next().?.shorts);
    try t.expectEqualStrings("value", args.nextAsValue().?);
    try t.expect(args.next() == null);
}

test "peek" {
    var args = mkArgs(&.{ "bin", "a" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualStrings("a", args.peek().?.value);
    try t.expectEqualStrings("a", args.peek().?.value);
    try t.expectEqualStrings("a", args.next().?.value);
    try t.expect(args.peeked == null);

    try t.expect(args.next() == null);
}

test "nextAsValue" {
    var args = mkArgs(&.{ "bin", "--a", "-b", "--", "c" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualStrings("--a", args.nextAsValue().?);
    try t.expectEqualStrings("-b", args.nextAsValue().?);
    try t.expectEqualStrings("c", args.nextAsValue().?);
    try t.expect(args.next() == null);
}

test "nextIfValue" {
    var args = mkArgs(&.{ "bin", "--a", "b", "--c", "--d" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "a" }, args.next().?.long);
    try t.expectEqualStrings("b", args.nextIfValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "c" }, args.next().?.long);
    try t.expect(args.nextIfValue() == null);
    try t.expectEqualDeep(Arg.Long{ .flag = "d" }, args.next().?.long);
    try t.expect(args.next() == null);
}

test "skip" {
    var args = mkArgs(&.{ "bin", "--flag1", "value", "--flag2" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "flag1" }, args.next().?.long);
    try t.expectEqual(true, args.skip());
    try t.expectEqualDeep(Arg.Long{ .flag = "flag2" }, args.next().?.long);
    try t.expectEqual(false, args.skip());
    try t.expect(args.next() == null);
}

test "reset" {
    var args = mkArgs(&.{ "bin", "a", "b", "c" });
    defer args.deinit();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualStrings("a", args.next().?.value);
    try t.expectEqualStrings("b", args.next().?.value);
    try t.expectEqualStrings("c", args.next().?.value);
    try t.expect(args.next() == null);

    args.reset();

    try t.expectEqualStrings("bin", args.nextAsValue().?);
    try t.expectEqualStrings("a", args.next().?.value);
    try t.expectEqualStrings("b", args.next().?.value);
    try t.expectEqualStrings("c", args.next().?.value);
    try t.expect(args.next() == null);
}

test "handles escapes: next" {
    var args = mkArgs(&.{ "bin", "--long", "-x", "value", "--", "--also", "-X", "another" });
    defer args.deinit();

    var escaping = args.handleEscape();

    try t.expectEqualStrings("bin", escaping.nextAsValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "long" }, escaping.next().?.long);
    try t.expectEqualDeep(Arg.Shorts{ .flags = "x" }, escaping.next().?.shorts);
    try t.expectEqualStrings("value", escaping.next().?.value);
    try t.expectEqualStrings("--also", escaping.next().?.value);
    try t.expectEqualStrings("-X", escaping.next().?.value);
    try t.expectEqualStrings("another", escaping.next().?.value);
    try t.expect(escaping.next() == null);
}

test "handles escapes: nextAsValue" {
    var args = mkArgs(&.{ "bin", "--long", "-x", "value", "--", "--also", "-X", "another" });
    defer args.deinit();

    var escaping = args.handleEscape();

    try t.expectEqualStrings("bin", escaping.nextAsValue().?);
    try t.expectEqualStrings("--long", escaping.nextAsValue().?);
    try t.expectEqualStrings("-x", escaping.nextAsValue().?);
    try t.expectEqualStrings("value", escaping.nextAsValue().?);
    try t.expectEqualStrings("--also", escaping.nextAsValue().?);
    try t.expectEqualStrings("-X", escaping.nextAsValue().?);
    try t.expectEqualStrings("another", escaping.nextAsValue().?);
    try t.expect(escaping.next() == null);
}

test "handles escapes: nextIfValue" {
    var args = mkArgs(&.{ "bin", "--long", "-x", "value", "--", "--also", "-X", "another" });
    defer args.deinit();

    var escaping = args.handleEscape();

    try t.expectEqualStrings("bin", escaping.nextAsValue().?);
    try t.expectEqual(null, escaping.nextIfValue());
    try t.expect(escaping.skip()); // --long
    try t.expectEqual(null, escaping.nextIfValue());
    try t.expect(escaping.skip()); // --x
    try t.expectEqualStrings("value", escaping.nextIfValue().?);
    try t.expectEqualStrings("--also", escaping.nextIfValue().?);
    try t.expectEqualStrings("-X", escaping.nextIfValue().?);
    try t.expectEqualStrings("another", escaping.nextIfValue().?);
    try t.expect(escaping.next() == null);
}

test "yields escapes: next" {
    var args = mkArgs(&.{ "bin", "--long", "-x", "value", "--", "--also", "-X", "another" });
    defer args.deinit();

    var escaping = args.yieldEscape();

    try t.expectEqualStrings("bin", escaping.nextAsValue().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "long" }, escaping.next().?.long);
    try t.expectEqualDeep(Arg.Shorts{ .flags = "x" }, escaping.next().?.shorts);
    try t.expectEqualStrings("value", escaping.next().?.value);
    try t.expectEqualDeep(.escape, escaping.next().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "also" }, escaping.next().?.long);
    try t.expectEqualDeep(Arg.Shorts{ .flags = "X" }, escaping.next().?.shorts);
    try t.expectEqualStrings("another", escaping.next().?.value);
    try t.expect(escaping.next() == null);
}

test "yields escapes: nextAsValue" {
    var args = mkArgs(&.{ "bin", "--long", "-x", "value", "--", "--also", "-X", "another" });
    defer args.deinit();

    var escaping = args.yieldEscape();

    try t.expectEqualStrings("bin", escaping.nextAsValue().?);
    try t.expectEqualStrings("--long", escaping.nextAsValue().?);
    try t.expectEqualStrings("-x", escaping.nextAsValue().?);
    try t.expectEqualStrings("value", escaping.nextAsValue().?);
    try t.expectEqualDeep(.escape, escaping.next().?);
    try t.expectEqualDeep(Arg.Long{ .flag = "also" }, escaping.next().?.long);
    try t.expectEqualDeep(Arg.Shorts{ .flags = "X" }, escaping.next().?.shorts);
    try t.expectEqualStrings("another", escaping.nextAsValue().?);
    try t.expect(escaping.next() == null);
}

test "yields escapes: nextIfValue" {
    var args = mkArgs(&.{ "bin", "--long", "-x", "value", "--", "--also", "-X", "another" });
    defer args.deinit();

    var escaping = args.yieldEscape();

    try t.expectEqualStrings("bin", escaping.nextAsValue().?);
    try t.expectEqual(null, escaping.nextIfValue());
    try t.expect(escaping.skip()); // --long
    try t.expectEqual(null, escaping.nextIfValue());
    try t.expect(escaping.skip()); // --x
    try t.expectEqualStrings("value", escaping.nextIfValue().?);
    try t.expect(escaping.skip()); // --
    try t.expect(escaping.skip()); // --also
    try t.expect(escaping.skip()); // -X
    try t.expectEqualStrings("another", escaping.nextIfValue().?);
    try t.expect(escaping.next() == null);
}

test "Force analysis" {
    comptime {
        t.refAllDecls(@This());
    }
}

const std = @import("std");
