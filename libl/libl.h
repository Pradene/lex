#ifndef LIB_H
#define LIB_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern int yylex(void);

/* Default global state */
extern char* yytext;
extern int   yyleng;
extern int   yylineno;
extern int   yycolumn;
extern FILE* yyin;

#endif