#include <pthread.h>

int pthread_mutex_consistent_np(pthread_mutex_t *mutex) {
    return pthread_mutex_consistent(mutex);
}

int pthread_mutexattr_setrobust_np(pthread_mutexattr_t *attr, int robustness) {
    return pthread_mutexattr_setrobust(attr, robustness);
}

