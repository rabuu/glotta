const std = @import("std");
const SourcePosition = @import("SourcePosition.zig");

const Token = @This();

tag: Tag,
pos: SourcePosition,

pub const keywords = std.StaticStringMap(Tag).initComptime(.{
    .{ "fn", .kw_fn },
    .{ "Int", .kw_int },
});

pub const Tag = enum {
    // keywords
    kw_fn,
    kw_int,

    // literals
    lit_int,

    ident,

    paren_open,
    paren_close,
    curly_open,
    curly_close,

    comma,
    colon,
    semicolon,
    assign,
    plus,

    // end of file
    eof,

    invalid,

    pub fn toString(self: Tag) []const u8 {
        return switch (self) {
            .kw_fn => "fn",
            .kw_int => "Int",

            .lit_int => "LITERAL-INT",

            .ident => "IDENTIFIER",

            .paren_open => "(",
            .paren_close => ")",
            .curly_open => "{",
            .curly_close => "}",

            .comma => ",",
            .colon => ":",
            .semicolon => ";",
            .assign => "=",
            .plus => "+",

            .eof => "EOF",
            .invalid => "INVALID",
        };
    }
};
