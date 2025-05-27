if exists("b:current_syntax")
	finish
end

syn keyword glottaKeyword fn
syn keyword glottaDataType Int Bool
syn keyword glottaConditional if then else
syn keyword glottaBoolean true false
syn match glottaNumber "-\=\<[0-9]*\>"
syn match glottaFloat "-\=\<[0-9]*\.[0-9]*\>"
syn region glottaComment start="//" end="\n"

hi def link glottaKeyword Keyword
hi def link glottaDataType Type
hi def link glottaConditional Conditional
hi def link glottaBoolean Boolean
hi def link glottaNumber Number
hi def link glottaFloat Float
hi def link glottaComment Comment

let b:current_syntax = "glotta"
