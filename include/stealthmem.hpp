#pragma once

#ifdef __KERNEL__

#include <linux/ioctl.h>
#include <linux/pid.h>

struct memory_params {
    pid_t pid;
    unsigned long addr;
    size_t size;
    void __user *buf;
};

#else

#include <fcntl.h>
#include <stddef.h>
#include <sys/ioctl.h>

struct memory_params {
    pid_t pid;
    unsigned long addr;
    size_t size;
    void *buf;
};

#endif

struct mouse_move {
    int x;
    int y;
};

// used for keyboard and mouse buttons
struct key_press {
    int key;
    unsigned char press;
};

#define IOCTL_MAGIC_READ 0xBC
#define IOCTL_MAGIC_WRITE 0xBD
#define IOCTL_MAGIC_MOUSE 0xBE
#define IOCTL_MAGIC_KEY 0xBF

#define IOCTL_READ_MEM _IOWR(IOCTL_MAGIC_READ, 1, struct memory_params)
#define IOCTL_WRITE_MEM _IOWR(IOCTL_MAGIC_WRITE, 1, struct memory_params)
#define IOCTL_MOUSE_MOVE _IOWR(IOCTL_MAGIC_MOUSE, 1, struct mouse_move)
#define IOCTL_KEY_PRESS _IOWR(IOCTL_MAGIC_KEY, 1, struct key_press)
