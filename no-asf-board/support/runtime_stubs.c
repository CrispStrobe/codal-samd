// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Freestanding C runtime hooks for the no-ASF board (no crt start files, no
// libnosys). Process exit and signals have no meaning here: they panic.
#include <errno.h>

void target_panic(int);

void *__dso_handle = 0;
void _init(void) {}
void _fini(void) {}

void _exit(int status)
{
    (void)status;
    target_panic(999);
    for (;;) {
    }
}

int _kill(int pid, int signal)
{
    (void)pid;
    (void)signal;
    errno = EINVAL;
    return -1;
}

int _getpid(void) { return 1; }
