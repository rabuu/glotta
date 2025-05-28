const std = @import("std");

const lexing = @import("./parsing/lexing/mod.zig");

pub fn main() !void {
    const allocator = std.heap.page_allocator;

    var args = std.process.args();
    _ = args.next();
    const path = args.next().?;

    std.debug.print("PATH: {s}\n--------------------\n", .{path});

    const max_size = std.math.maxInt(usize);

    const source_bare = try std.fs.cwd().readFileAlloc(allocator, path, max_size);
    const source = try allocator.dupeZ(u8, source_bare);
    allocator.free(source_bare);
    defer allocator.free(source);

    std.debug.print("{s}\n--------------------\n", .{source});

    var lexer = lexing.Lexer.init(source);

    var tok: lexing.Token = undefined;
    while (tok.tag != .eof) {
        tok = lexer.next();
        std.debug.print("{s}\n", .{tok.tag.toString()});
    }
}
