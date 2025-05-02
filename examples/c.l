%{
  #include <stdio.h>
%}

DIGIT       [0-9]
NUMBER      {DIGIT}+\.?{DIGIT}*
ID          [a-zA-Z][a-zA-Z0-9_]*
STRING      \"[^\"]*\"
COMMENT     \/\/.*

%%

"if"        { printf("KEYWORD: %s\n", yytext); }
"else"      { printf("KEYWORD: %s\n", yytext); }
"while"     { printf("KEYWORD: %s\n", yytext); }
"for"       { printf("KEYWORD: %s\n", yytext); }
"int"       { printf("TYPE: %s\n", yytext); }
"float"     { printf("TYPE: %s\n", yytext); }
"char"      { printf("TYPE: %s\n", yytext); }
"return"    { printf("KEYWORD: %s\n", yytext); }

{ID}        { printf("IDENTIFIER: %s\n", yytext); }
{NUMBER}    { printf("NUMBER: %s\n", yytext); }
{STRING}    { printf("STRING: %s\n", yytext); }
{COMMENT}   { printf("COMMENT: %s\n", yytext); }

"+"         { printf("OPERATOR: %s\n", yytext); }
"-"         { printf("OPERATOR: %s\n", yytext); }
"*"         { printf("OPERATOR: %s\n", yytext); }
"/"         { printf("OPERATOR: %s\n", yytext); }
"="         { printf("ASSIGNMENT: %s\n", yytext); }
"=="        { printf("OPERATOR: %s\n", yytext); }
"!="        { printf("OPERATOR: %s\n", yytext); }
"<"         { printf("OPERATOR: %s\n", yytext); }
">"         { printf("OPERATOR: %s\n", yytext); }
"<="        { printf("OPERATOR: %s\n", yytext); }
">="        { printf("OPERATOR: %s\n", yytext); }
"&&"        { printf("OPERATOR: %s\n", yytext); }
"||"        { printf("OPERATOR: %s\n", yytext); }

"{"         { printf("OPEN_BRACE\n"); }
"}"         { printf("CLOSE_BRACE\n"); }
"("         { printf("OPEN_PAREN\n"); }
")"         { printf("CLOSE_PAREN\n"); }
"["         { printf("OPEN_BRACKET\n"); }
"]"         { printf("CLOSE_BRACKET\n"); }
";"         { printf("SEMICOLON\n"); }
","         { printf("COMMA\n"); }

"\n"        printf("NEWLINE\n");
"\t"        printf("TAB\n");
[[:blank:]] { /* ignore whitespaces */ }
.           { printf("UNRECOGNIZED: %s\n", yytext); }

