%{
#include <stdio.h>
%}

DIGIT       [0-9]
INT         -?{DIGIT}+
FLOAT       -?{DIGIT}+\.{DIGIT}+([eE][+-]?{DIGIT}+)?
STRING      \"(\\.|[^"\\])*\"
WS          [ \t\n\r]

%%

"{"         { printf("OPEN_OBJECT\n"); }
"}"         { printf("CLOSE_OBJECT\n"); }
"["         { printf("OPEN_ARRAY\n"); }
"]"         { printf("CLOSE_ARRAY\n"); }
":"         { printf("COLON\n"); }
","         { printf("COMMA\n"); }

"true"      { printf("BOOLEAN: %s\n", yytext); }
"false"     { printf("BOOLEAN: %s\n", yytext); }
"null"      { printf("NULL\n"); }

{INT}       { printf("INTEGER: %s\n", yytext); }
{FLOAT}     { printf("FLOAT: %s\n", yytext); }
{STRING}    { 
    /* Remove quotes for display */
    yytext[yyleng-1] = '\0';
    printf("STRING: %s\n", yytext+1);
    yytext[yyleng-1] = '\"'; /* restore for proper memory management */
}

{WS}+       { /* ignore whitespace */ }
.           { printf("INVALID: %s\n", yytext); }

