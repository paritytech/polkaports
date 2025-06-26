#include "syscall.h"
#include "locale_impl.h"
#include "pthread_impl.h"
#include "pthread_arch.h"

long _syscall_polkavm(long n, long a, long b, long c, long d, long e, long f)
{
    return pvm_syscall(n, a, b, c, d, e, f);
}

static struct pthread tls = {
    .locale = &__libc.global_locale,
};

uintptr_t __get_tp() {
    return (uintptr_t) &tls;
}

int __set_thread_area(void *p) {
    return 1;
}
