if exists("b:current_syntax")
	finish
end

syn keyword glottaKeyword fun val var typ alias variant struct pub use match
syn keyword glottaSpecial self
syn keyword glottaDataType Unit Bool Nat Int Byte Float Str Slice Option
syn keyword glottaUnitExpression unit
syn keyword glottaConditional if then else
syn keyword glottaBoolean true false
syn match glottaNumber "-\=\<[0-9]*\>"
syn match glottaFloat "-\=\<[0-9]*\.[0-9]*\>"
syn region glottaComment start="//" end="\n"

hi def link glottaKeyword Keyword
hi def link glottaDataType Type
hi def link glottaSpecial Special
hi def link glottaUnitExpression Special
hi def link glottaConditional Conditional
hi def link glottaBoolean Boolean
hi def link glottaNumber Number
hi def link glottaFloat Float
hi def link glottaComment Comment

let b:current_syntax = "glotta"
