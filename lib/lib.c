#include "lib.h"

char* yytext = NULL;
int   yyleng = 0;
int   yylineno = 1;
int   yycolumn = 0;
FILE* yyin = NULL;

/* Weak default implementations */
__attribute__((weak)) int yywrap(void) { return 1; }

/* Weak default main implementation */
__attribute__((weak)) int main(int argc, char* argv[]) {
    yyin = stdin;
    if (argc > 1) {
        yyin = fopen(argv[1], "r");
        if (yyin != 0) return 1;
    }

    while(yylex() != 0) ;
    if (yyin != stdin) fclose(yyin);
    return 0;
}
