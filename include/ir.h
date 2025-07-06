#ifndef IR_H_
#define IR_H_

#include <stddef.h>
#include <stdint.h>

typedef enum {
	IR_TYPE_I32,
} IR_Type;

typedef struct {
	size_t id;
	IR_Type type;
} IR_Variable;

typedef enum {
	IR_OP_LOAD,
	IR_OP_STORE,
	IR_OP_ALLOC,

	IR_OP_ADD,

	IR_OP_CALL,
	IR_OP_RETURN,
} IR_Opcode;

typedef struct {
	IR_Opcode op;
	union {
		int32_t i32;
		IR_Variable var;
	} payload;
} IR_Instr;

#endif // !IR_H_
