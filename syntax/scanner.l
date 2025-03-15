int|char                    printf("KEYWORD: %s\n", yytext);
[0-9]+                      printf("NUMBER: %s\n", yytext);
[a-zA-Z_][a-zA-Z0-9_]*       printf("IDENTIFIER: %s\n", yytext);