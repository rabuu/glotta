if exists("b:current_syntax")
	finish
end

syn region glottaComment start="//" end="\n"

syn region glottaString start='"' end='"' skip='\\.' contains=glottaStringEscape oneline
syn match glottaStringEscape "\\[ntr\\\"']" contained
syn match glottaStringEscape "\\x[0-9a-fA-F]\{2}" contained
syn match glottaStringEscape "\\u[0-9a-fA-F]\{4}" contained

syn match glottaNumber "-\=\<[0-9]*\>"
syn match glottaFloat "-\=\<[0-9]*\.[0-9]*\>"
syn keyword glottaBoolean true false

syn keyword glottaConditional if then else match
syn keyword glottaRepeat loop
syn match glottaOperator "[+*-/<>:=!]"
syn keyword glottaKeyword fun val var typ for pub use return

syn keyword glottaType Unit Bool Byte Nat Int Float Str
syn keyword glottaStructure struct variant alias

syn keyword glottaSpecial self unit
syn match glottaDelimiter "[,;.]"

hi def link glottaComment Comment
hi def link glottaString String
hi def link glottaStringEscape SpecialChar
hi def link glottaNumber Number
hi def link glottaFloat Float
hi def link glottaBoolean Boolean
hi def link glottaConditional Conditional
hi def link glottaRepeat Repeat
hi def link glottaOperator Operator
hi def link glottaKeyword Keyword
hi def link glottaType Type
hi def link glottaStructure Structure
hi def link glottaSpecial Special
hi def link glottaDelimiter Delimiter

let b:current_syntax = "glotta"
