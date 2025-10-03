" Vim syntax file
" Language: Rono
" Maintainer: Rono Language Team
" Latest Revision: 2024

if exists("b:current_syntax")
  finish
endif

" Keywords
syn keyword ronoKeyword chif fn struct var list array fn_for
syn keyword ronoControl if else for while break continue ret import
syn keyword ronoConstant true false nil
syn keyword ronoSelf self

" Types
syn keyword ronoType int float bool str pointer
syn match ronoUserType '\<[A-Z][a-zA-Z0-9_]*\>'

" Comments
syn match ronoComment "//.*$"
syn region ronoBlockComment start="/\*" end="\*/"

" Strings
syn region ronoString start='"' end='"' contains=ronoStringInterpolation,ronoEscape
syn match ronoEscape contained '\\.'
syn region ronoStringInterpolation contained start='{' end='}' contains=ronoNumber,ronoKeyword

" Numbers
syn match ronoNumber '\<\d\+\>'
syn match ronoFloat '\<\d\+\.\d\+\>'

" Functions
syn match ronoFunction '\<[a-z_][a-zA-Z0-9_]*\ze\s*('
syn match ronoBuiltinFunction '\<\(con\.out\|con\.in\|randi\|randf\|rands\|http\.get\|http\.post\)\>'

" Operators
syn match ronoOperator '[+\-*/%]'
syn match ronoOperator '[=!<>]=\?'
syn match ronoOperator '&&\|||'
syn match ronoOperator '[&*]'

" Delimiters
syn match ronoDelimiter '[{}()\[\],;]'

" Highlighting
hi def link ronoKeyword Keyword
hi def link ronoControl Conditional
hi def link ronoConstant Constant
hi def link ronoSelf Special
hi def link ronoType Type
hi def link ronoUserType Type
hi def link ronoComment Comment
hi def link ronoBlockComment Comment
hi def link ronoString String
hi def link ronoStringInterpolation Special
hi def link ronoEscape SpecialChar
hi def link ronoNumber Number
hi def link ronoFloat Float
hi def link ronoFunction Function
hi def link ronoBuiltinFunction Function
hi def link ronoOperator Operator
hi def link ronoDelimiter Delimiter

let b:current_syntax = "rono"