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
