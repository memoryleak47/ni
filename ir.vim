" ~/.vim/syntax/ir.vim
if exists("b:current_syntax")
  finish
endif

syntax keyword irKeyword proc jmp exit panic main print fail
syntax match irComment "#.*$"
syntax region irString start=+"+ skip=+\\."+ end=+"+
syntax match irNumber /\v(^|[^A-Za-z0-9_])\zs\d+(\.\d+)?\ze([^A-Za-z0-9_]|$)/
syntax match irOperator /[+\-\*%=;:(),]/

syntax match irHole /@/
syntax match irAlloc /{}/
syntax match irVar /%[A-Za-z_][A-Za-z0-9_]*/

" link highlight groups (use 'default' so user colors are preserved)
highlight default link irKeyword Keyword
highlight default link irComment Comment
highlight default link irString String
highlight default link irNumber Number
highlight default link irVar Type
highlight default link irHole Special
highlight default link irAlloc Special
highlight default link irOperator Operator

let b:current_syntax = "ir"

