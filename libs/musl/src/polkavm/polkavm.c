#include "syscall.h"

long _syscall_polkavm(long n, long a, long b, long c, long d, long e, long f)
{
    return pvm_syscall(n, a, b, c, d, e, f);
}

static unsigned char TLS[256];

unsigned long __get_tp() {
    return (unsigned long)TLS;
}

int __set_thread_area(void *p) {
    return 1;
}
