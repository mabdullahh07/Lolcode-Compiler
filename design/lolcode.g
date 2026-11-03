grammar lolcode;

// lexical definitions
HAI           : '#' ('H' | 'h') ('A' | 'a') ('I' | 'i') ;
KBYE          : '#' ('K' | 'k') ('B' | 'b') ('Y' | 'y') ('E' | 'e') ;
OBTW          : '#' ('O' | 'o') ('B' | 'b') ('T' | 't') ('W' | 'w') ;
TLDR          : '#' ('T' | 't') ('L' | 'l') ('D' | 'd') ('R' | 'r') ;
MAEK_HEAD     : '#' ('M' | 'm') ('A' | 'a') ('E' | 'e') ('K' | 'k') (' ' | '\t')+ ('H' | 'h') ('E' | 'e') ('A' | 'a') ('D' | 'd') ;
MAEK_PARAGRAF : '#' ('M' | 'm') ('A' | 'a') ('E' | 'e') ('K' | 'k') (' ' | '\t')+ ('P' | 'p') ('A' | 'a') ('R' | 'r') ('A' | 'a') ('G' | 'g') ('R' | 'r') ('A' | 'a') ('F' | 'f') ;
MAEK_LIST     : '#' ('M' | 'm') ('A' | 'a') ('E' | 'e') ('K' | 'k') (' ' | '\t')+ ('L' | 'l') ('I' | 'i') ('S' | 's') ('T' | 't') ;
MKAY          : '#' ('M' | 'm') ('K' | 'k') ('A' | 'a') ('Y' | 'y') ;
GIMMEH_TITLE  : '#' ('G' | 'g') ('I' | 'i') ('M' | 'm') ('M' | 'm') ('E' | 'e') ('H' | 'h') (' ' | '\t')+ ('T' | 't') ('I' | 'i') ('T' | 't') ('L' | 'l') ('E' | 'e') ;
GIMMEH_BOLD   : '#' ('G' | 'g') ('I' | 'i') ('M' | 'm') ('M' | 'm') ('E' | 'e') ('H' | 'h') (' ' | '\t')+ ('B' | 'b') ('O' | 'o') ('L' | 'l') ('D' | 'd') ;
GIMMEH_ITEM   : '#' ('G' | 'g') ('I' | 'i') ('M' | 'm') ('M' | 'm') ('E' | 'e') ('H' | 'h') (' ' | '\t')+ ('I' | 'i') ('T' | 't') ('E' | 'e') ('M' | 'm') ;
GIMMEH_LINX   : '#' ('G' | 'g') ('I' | 'i') ('M' | 'm') ('M' | 'm') ('E' | 'e') ('H' | 'h') (' ' | '\t')+ ('L' | 'l') ('I' | 'i') ('N' | 'n') ('X' | 'x') ;
GIMMEH_VAR    : '#' ('G' | 'g') ('I' | 'i') ('M' | 'm') ('M' | 'm') ('E' | 'e') ('H' | 'h') (' ' | '\t')+ ('V' | 'v') ('A' | 'a') ('R' | 'r') ;
OIC           : '#' ('O' | 'o') ('I' | 'i') ('C' | 'c') ;
NEWLINE       : '#' ('N' | 'n') ('E' | 'e') ('W' | 'w') ('L' | 'l') ('I' | 'i') ('N' | 'n') ('E' | 'e') ;
IHAZ          : '#' ('I' | 'i') ('H' | 'h') ('A' | 'a') ('Z' | 'z') ;
ITIZ          : '#' ('I' | 'i') ('T' | 't') ('I' | 'i') ('Z' | 'z') ;

// Plain text: starts with an allowed character, then may continue with allowed characters or spaces/tabs/newlines, up to (not including) the next '#'.
TEXT          : ('A'..'Z' | 'a'..'z' | '0'..'9' | ',' | '.' | '"' | ':' | '?' | '!' | '%' | '=' | '/')
                ('A'..'Z' | 'a'..'z' | '0'..'9' | ',' | '.' | '"' | ':' | '?' | '!' | '%' | '=' | '/' | ' ' | '\t' | '\r' | '\n')* ;

WS            : (' ' | '\t' | '\r' | '\n') {$channel=HIDDEN;} ;


// syntax definitions

lolcode         : HAI comment head variable_define body KBYE EOF ;

comment         : OBTW text TLDR comment | ;

head            : MAEK_HEAD title MKAY comment |;

title           : GIMMEH_TITLE text OIC ;

variable_define : IHAZ text ITIZ text MKAY comment variable_define | ;

body            : paragraph body | bold body | list body | link body | newline body | variable_use body | text body | ;

paragraph       : MAEK_PARAGRAF variable_define inner_paragraph MKAY comment ;

inner_paragraph : text inner_paragraph | bold inner_paragraph | list inner_paragraph | link inner_paragraph | newline inner_paragraph | variable_use inner_paragraph | ;

bold            : GIMMEH_BOLD variable_define inner_text OIC ;

inner_text      : text inner_text | variable_use inner_text | ;

list            : MAEK_LIST GIMMEH_ITEM variable_define inner_list OIC list_items MKAY comment ;

list_items      : GIMMEH_ITEM variable_define inner_list OIC list_items | ;

inner_list      : text inner_list | bold inner_list | variable_use inner_list | ;

link            : GIMMEH_LINX text OIC ;

newline         : NEWLINE comment ;

variable_use    : GIMMEH_VAR text OIC ;

text            : TEXT ;
