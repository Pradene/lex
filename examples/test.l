%%

[a-zA-Z]+[ ]+[a-zA-Z]+    { printf("%s\n", yytext); REJECT; }
[a-zA-Z]+[ ]*             { /* Do nothing */ }