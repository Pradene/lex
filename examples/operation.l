DIGIT       [0-9]+\.?[0-9]*

%%

{DIGIT}     printf("NUMBER: %s\n", yytext);
"+"         |
"-"         printf("OPERATOR: %s\n", yytext);
[*/]        {
    printf("OPERATOR: %s\n", yytext);
}
"="         ECHO;
"("         printf("OPEN PARENTHESIS\n");
")"         printf("CLOSED PARENTHESIS\n");
"\n"        printf("NEWLINE\n");
[[:blank:]] // ignore whitespaces

%%

int main(int argc, char **argv) {
    yyin = stdin;
    if (argc > 1) {
        yyin = fopen(argv[1], "r");
        if (yyin == NULL) {
            return 1;
        }
    }
    
    while (yylex() != 0) ;
    return 0;
}