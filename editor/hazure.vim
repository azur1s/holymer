" Vim syntax file
" Language: Hazure

if exists("b:current_syntax")
    finish
endif

set iskeyword=a-z,A-Z,_

" Todos
syn keyword hazureTodos TODO XXX FIXME NOTE

" Language keywords
syn keyword hazureKeywords let mut fun do end if then else case of return pub
syn keyword hazureTypes    int string bool void vec_int vec_string vec_bool

" Comments
syn region  hazureCommentLine  start="--" end="$"  contains=hazureTodos
syn region  hazureCommentBlock start="-{" end="}-" contains=hazureTodos

" Strings
syn region  hazureString       start=/\v"/  skip=/\v\\./ end=/\v"/

" Numbers
syn region  hazureNumber       start=/\s\d/ skip=/\d/    end=/\s/

" Set hilighting
hi def link hazureTodos        Todo
hi def link hazureKeywords     Keyword
hi def link hazureTypes        Types
hi def link hazureCommentLine  Comment
hi def link hazureCommentBlock Comment
hi def link hazureString       String
hi def link hazureNumber       Number

let b:current_syntax="hazure"
