CC = clang
CFLAGS = -Wall -Wextra -std=c23 -Iinclude
SRC = $(wildcard src/*.c src/**/*.c)
INC = $(wildcard include/*.h include/**/*.h)
OBJ = $(SRC:.c=.o)
TARGET = glotta

$(TARGET): $(OBJ)
	$(CC) $(CFLAGS) -o $@ $^

clean:
	rm -f $(OBJ) $(TARGET)

fmt:
	clang-format -i -- $(SRC) $(INC)

.PHONY: clean fmt
