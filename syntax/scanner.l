int|char        printf("KEYWORD: %s\n", yytext);
[0-9]+          printf("NUMBER: %s\n", yytext);
[a-zA-Z_]+      printf("IDENTIFIER: %s\n", yytext);