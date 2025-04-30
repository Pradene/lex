# Compiler and flags
CC = cc
CFLAGS = -Wall -Wextra -Werror -Ilib
LDFLAGS = -L. -ll

# Project structure
LEX = ./lex
LIB_SRC = lib/lib.c
LIB_HEADER = lib/lib.h
LIB_OBJ = $(LIB_SRC:.c=.o)
LIB_NAME = libl.a

LEX_SRC = examples/operation.l
LEX_GEN = lex.yy.c
TARGET = lex

all: $(TARGET)

$(TARGET): $(LEX_GEN) $(LIB_NAME)
	$(CC) $(CFLAGS) $< $(LDFLAGS) -o $@

$(LIB_NAME): $(LIB_OBJ)
	ar rcs $@ $^

$(LEX_GEN): $(LEX_SRC)
	cargo build
	./target/debug/lex $<

# Track header dependencies for library
$(LIB_OBJ): $(LIB_HEADER)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

re: fclean all

clean:
	rm -f $(LEX_GEN) $(LIB_OBJ) $(LIB_NAME)
	
fclean: clean
	rm -f $(TARGET)

.PHONY: all re clean fclean
