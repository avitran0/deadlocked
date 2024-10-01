#ifndef MOUSE_H
#define MOUSE_H

#include "types.h"

bool setup_mouse(void);
void close_mouse(void);
void move_mouse(i32 x, i32 y);

#endif