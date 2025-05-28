// see https://github.com/ziglang/zig/blob/master/lib/std/zig/tokenizer.zig

const std = @import("std");
const Token = @import("Token.zig");

const Lexer = @This();

buffer: [:0]const u8,
index: usize,

pub fn dump(self: *Lexer, token: *const Token) void {
    std.debug.print("{s} \"{s}\"\n", .{ @tagName(token.tag), self.buffer[token.pos.start..token.pos.end] });
}

pub fn init(buffer: [:0]const u8) Lexer {
    return .{
        .buffer = buffer,

        // NOTE: We may want to skip a potential UTF-8 BOM here
        .index = 0,
    };
}

const State = enum {
    start,
    ident,
    int,
    invalid,
};

pub fn next(self: *Lexer) Token {
    var result: Token = .{
        .tag = undefined,
        .pos = .{
            .start = self.index,
            .end = undefined,
        },
    };

    state: switch (State.start) {
        .start => switch (self.buffer[self.index]) {
            0 => {
                if (self.index == self.buffer.len) {
                    return .{
                        .tag = .eof,
                        .pos = .{
                            .start = self.index,
                            .end = self.index,
                        },
                    };
                } else {
                    continue :state .invalid;
                }
            },
            ' ', '\n', '\t', '\r' => {
                self.index += 1;
                result.pos.start = self.index;
                continue :state .start;
            },
            'a'...'z', 'A'...'Z', '_' => {
                result.tag = .ident;
                continue :state .ident;
            },
            '0'...'9' => {
                result.tag = .lit_int;
                self.index += 1;
                continue :state .int;
            },
            '(' => {
                result.tag = .paren_open;
                self.index += 1;
            },
            ')' => {
                result.tag = .paren_close;
                self.index += 1;
            },
            '{' => {
                result.tag = .curly_open;
                self.index += 1;
            },
            '}' => {
                result.tag = .curly_close;
                self.index += 1;
            },
            ',' => {
                result.tag = .comma;
                self.index += 1;
            },
            ':' => {
                result.tag = .colon;
                self.index += 1;
            },
            ';' => {
                result.tag = .semicolon;
                self.index += 1;
            },
            '=' => {
                result.tag = .assign;
                self.index += 1;
            },
            '+' => {
                result.tag = .plus;
                self.index += 1;
            },
            else => continue :state .invalid,
        },
        .ident => {
            self.index += 1;
            switch (self.buffer[self.index]) {
                'a'...'z', 'A'...'Z', '_', '0'...'9' => continue :state .ident,
                else => {
                    const ident = self.buffer[result.pos.start..self.index];
                    if (Token.keywords.get(ident)) |tag| {
                        result.tag = tag;
                    }
                },
            }
        },
        .int => switch (self.buffer[self.index]) {
            '_', '0'...'9' => {
                self.index += 1;
                continue :state .int;
            },
            else => {},
        },
        .invalid => {
            self.index += 1;
            switch (self.buffer[self.index]) {
                0 => if (self.index == self.buffer.len) {
                    result.tag = .invalid;
                } else {
                    continue :state .invalid;
                },
                '\n' => result.tag = .invalid,
                else => continue :state .invalid,
            }
        },
    }

    result.pos.end = self.index;
    return result;
}

test "keywords" {
    try testTokenize("fn Int fn", &.{ .kw_fn, .kw_int });
}

fn testTokenize(source: [:0]const u8, expected_token_tags: []const Token.Tag) !void {
    var tokenizer = Lexer.init(source);
    for (expected_token_tags) |expected_token_tag| {
        const token = tokenizer.next();
        try std.testing.expectEqual(expected_token_tag, token.tag);
    }

    const last_token = tokenizer.next();
    try std.testing.expectEqual(Token.Tag.eof, last_token.tag);
    try std.testing.expectEqual(source.len, last_token.loc.start);
    try std.testing.expectEqual(source.len, last_token.loc.end);
}
