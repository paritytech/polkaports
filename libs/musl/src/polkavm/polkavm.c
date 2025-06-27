#include "syscall.h"
#include "locale_impl.h"
#include "pthread_impl.h"
#include "pthread_arch.h"

long _syscall_polkavm(long n, long a, long b, long c, long d, long e, long f)
{
    return pvm_syscall(n, a, b, c, d, e, f);
}

static uintptr_t dtv[1] = { 0 };

static struct pthread tls = {
    .tid = 1,
    .locale = &__libc.global_locale,
    .self = &tls,
    .prev = &tls,
    .next = &tls,
    .dtv = &dtv,
};

uintptr_t __get_tp() {
    return (uintptr_t) &tls;
}

int __set_thread_area(void *p) {
    return 1;
}
