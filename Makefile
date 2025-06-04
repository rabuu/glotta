CC = clang
CFLAGS = -Wall -Wextra -std=c23 -Iinclude
SRC = $(wildcard src/*.c)
OBJ = $(SRC:.c=.o)
TARGET = glotta

$(TARGET): $(OBJ)
	$(CC) $(CFLAGS) -o $@ $^

clean:
	rm -f src/*.o $(TARGET)

fmt:
	clang-format -i -- src/*.c include/*.h

.PHONY: clean fmt
