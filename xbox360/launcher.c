/*
 * RustCraft Xbox 360 Native Dashboard Launcher & Diagnostic USB Logger
 * Target: Microsoft Xbox 360 (Xenon 3.2 GHz PowerPC Tri-Core)
 * Environment: Aurora Dashboard / XeXMenu / DashLaunch / Xenia Canary (xboxkrnl.exe)
 *
 * Imports kernel primitives directly from xboxkrnl.exe via XEX2 ordinal import thunk table.
 */

typedef unsigned int   u32;
typedef unsigned short u16;
typedef unsigned char  u8;
typedef long long      i64;
typedef unsigned long long u64;

// NT / Xbox Kernel Structures
typedef struct _ANSI_STRING {
    u16 Length;
    u16 MaximumLength;
    char *Buffer;
} ANSI_STRING;

typedef struct _OBJECT_ATTRIBUTES {
    void *RootDirectory;
    ANSI_STRING *ObjectName;
    u32 Attributes;
} OBJECT_ATTRIBUTES;

typedef struct _IO_STATUS_BLOCK {
    union {
        u32 Status;
        void *Pointer;
    };
    u32 Information;
} IO_STATUS_BLOCK;

typedef struct _LARGE_INTEGER {
    union {
        struct {
            u32 LowPart;
            int HighPart;
        };
        i64 QuadPart;
    };
} LARGE_INTEGER;

// ============================================================================
// 1. Kernel Import Descriptors (.data.imports at 0x82002000)
// Format: 0x00000000 | Ordinal (record_type = 0)
// ============================================================================
__attribute__((section(".data.imports"), used, aligned(4))) u32 imp_desc_DbgPrint = 0x00000003;
__attribute__((section(".data.imports"), used, aligned(4))) u32 imp_desc_ExTerminateThread = 0x00000019;
__attribute__((section(".data.imports"), used, aligned(4))) u32 imp_desc_HalReturnToFirmware = 0x00000028;
__attribute__((section(".data.imports"), used, aligned(4))) u32 imp_desc_KeDelayExecutionThread = 0x0000005A;
__attribute__((section(".data.imports"), used, aligned(4))) u32 imp_desc_NtClose = 0x000000CF;
__attribute__((section(".data.imports"), used, aligned(4))) u32 imp_desc_NtCreateFile = 0x000000D2;
__attribute__((section(".data.imports"), used, aligned(4))) u32 imp_desc_NtWriteFile = 0x000000FF;
__attribute__((section(".data.imports"), used, aligned(4))) u32 imp_desc_RtlInitAnsiString = 0x0000012C;

// ============================================================================
// 2. Kernel Import Thunks (.text.thunks in executable code)
// Format: 0x01000000 | Ordinal (record_type = 1, 16 bytes each)
// Patched at load time with `sc 2; blr` (Xenia) or branch trampoline (real Xbox)
// ============================================================================
__attribute__((section(".text.thunks"), naked, aligned(16)))
void DbgPrint(const char *fmt, ...) {
    __asm__ volatile (
        ".long 0x01000003\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
    );
}

__attribute__((section(".text.thunks"), naked, aligned(16)))
void ExTerminateThread(u32 ExitCode) {
    __asm__ volatile (
        ".long 0x01000019\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
    );
}

__attribute__((section(".text.thunks"), naked, aligned(16)))
void HalReturnToFirmware(u32 Routine) {
    __asm__ volatile (
        ".long 0x01000028\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
    );
}

__attribute__((section(".text.thunks"), naked, aligned(16)))
void KeDelayExecutionThread(int WaitMode, int Alertable, LARGE_INTEGER *Interval) {
    __asm__ volatile (
        ".long 0x0100005A\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
    );
}

__attribute__((section(".text.thunks"), naked, aligned(16)))
u32 NtClose(void *Handle) {
    __asm__ volatile (
        ".long 0x010000CF\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
    );
}

__attribute__((section(".text.thunks"), naked, aligned(16)))
u32 NtCreateFile(
    void **FileHandle,
    u32 DesiredAccess,
    OBJECT_ATTRIBUTES *ObjectAttributes,
    IO_STATUS_BLOCK *IoStatusBlock,
    void *AllocationSize,
    u32 FileAttributes,
    u32 ShareAccess,
    u32 CreateDisposition,
    u32 CreateOptions
) {
    __asm__ volatile (
        ".long 0x010000D2\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
    );
}

__attribute__((section(".text.thunks"), naked, aligned(16)))
u32 NtWriteFile(
    void *FileHandle,
    void *Event,
    void *ApcRoutine,
    void *ApcContext,
    IO_STATUS_BLOCK *IoStatusBlock,
    const void *Buffer,
    u32 Length,
    LARGE_INTEGER *ByteOffset
) {
    __asm__ volatile (
        ".long 0x010000FF\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
    );
}

__attribute__((section(".text.thunks"), naked, aligned(16)))
void RtlInitAnsiString(ANSI_STRING *DestinationString, const char *SourceString) {
    __asm__ volatile (
        ".long 0x0100012C\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
        ".long 0x00000000\n\t"
    );
}

// ============================================================================
// 3. String & Memory Utilities
// ============================================================================
void *memcpy(void *dest, const void *src, u32 n) {
    u8 *d = (u8 *)dest;
    const u8 *s = (const u8 *)src;
    for (u32 i = 0; i < n; i++) d[i] = s[i];
    return dest;
}

void *memmove(void *dest, const void *src, u32 n) {
    u8 *d = (u8 *)dest;
    const u8 *s = (const u8 *)src;
    if (d < s) {
        for (u32 i = 0; i < n; i++) d[i] = s[i];
    } else if (d > s) {
        for (u32 i = n; i > 0; i--) d[i - 1] = s[i - 1];
    }
    return dest;
}

void *memset(void *s, int c, u32 n) {
    u8 *p = (u8 *)s;
    for (u32 i = 0; i < n; i++) p[i] = (u8)c;
    return s;
}

int memcmp(const void *s1, const void *s2, u32 n) {
    const u8 *p1 = (const u8 *)s1;
    const u8 *p2 = (const u8 *)s2;
    for (u32 i = 0; i < n; i++) {
        if (p1[i] != p2[i]) return (int)p1[i] - (int)p2[i];
    }
    return 0;
}

int bcmp(const void *s1, const void *s2, u32 n) {
    return memcmp(s1, s2, n);
}

u32 strlen(const char *s) {
    u32 len = 0;
    while (*s++) len++;
    return len;
}

static u32 str_len(const char *s) {
    return strlen(s);
}

// Math primitives for Xenon PowerPC (freestanding runtime)
#define RC_PI 3.14159265358979323846f
#define RC_TWO_PI 6.28318530717958647692f

float fabsf(float x) {
    union { float f; u32 u; } v;
    v.f = x;
    v.u &= 0x7FFFFFFF;
    return v.f;
}

float sqrtf(float x) {
    if (x <= 0.0f) return 0.0f;
    float res;
    __asm__ volatile ("fsqrts %0, %1" : "=f"(res) : "f"(x));
    return res;
}

float sinf(float x) {
    if (x < -RC_PI || x > RC_PI) {
        float q = x * (1.0f / RC_TWO_PI);
        int iq = (int)q;
        x -= (float)iq * RC_TWO_PI;
        while (x > RC_PI) x -= RC_TWO_PI;
        while (x < -RC_PI) x += RC_TWO_PI;
    }
    float x2 = x * x;
    float x3 = x * x2;
    float x5 = x3 * x2;
    float x7 = x5 * x2;
    float x9 = x7 * x2;
    return x - (x3 * 0.16666666666666666f)
             + (x5 * 0.008333333333333333f)
             - (x7 * 0.0001984126984126984f)
             + (x9 * 0.0000027557319223985f);
}

float cosf(float x) {
    return sinf(x + (RC_PI * 0.5f));
}

void sincosf(float x, float *s, float *c) {
    if (s) *s = sinf(x);
    if (c) *c = cosf(x);
}

float floorf(float x) {
    int i = (int)x;
    return (float)(x < (float)i ? i - 1 : i);
}

float ceilf(float x) {
    int i = (int)x;
    return (float)(x > (float)i ? i + 1 : i);
}

float tanf(float x) {
    float c = cosf(x);
    if (fabsf(c) < 1e-6f) return c >= 0.0f ? 1e6f : -1e6f;
    return sinf(x) / c;
}

float atan2f(float y, float x) {
    if (x == 0.0f) {
        if (y > 0.0f) return RC_PI * 0.5f;
        if (y < 0.0f) return -RC_PI * 0.5f;
        return 0.0f;
    }
    float z = y / x;
    float abs_z = fabsf(z);
    float a;
    if (abs_z <= 1.0f) {
        float z2 = z * z;
        a = z * (1.0f - z2 * (0.33333333f - z2 * (0.2f - z2 * 0.14285714f)));
    } else {
        float inv = 1.0f / z;
        float inv2 = inv * inv;
        a = (z > 0.0f ? (RC_PI * 0.5f) : (-RC_PI * 0.5f))
            - inv * (1.0f - inv2 * (0.33333333f - inv2 * (0.2f - inv2 * 0.14285714f)));
    }
    if (x < 0.0f) {
        if (y >= 0.0f) a += RC_PI;
        else a -= RC_PI;
    }
    return a;
}

float expf(float x) {
    if (x > 88.0f) x = 88.0f;
    if (x < -88.0f) return 0.0f;
    int k = (int)(x * 1.4426950408889634f);
    float r = x - (float)k * 0.6931471805599453f;
    float r2 = r * r;
    float r3 = r2 * r;
    float r4 = r3 * r;
    float exp_r = 1.0f + r + (r2 * 0.5f) + (r3 * 0.16666667f) + (r4 * 0.04166667f);
    float scale = 1.0f;
    if (k > 0) {
        for (int i = 0; i < k; i++) scale *= 2.0f;
    } else if (k < 0) {
        for (int i = 0; i < -k; i++) scale *= 0.5f;
    }
    return exp_r * scale;
}

// System shims & Allocator for freestanding Xbox 360 runtime
static int g_errno_val = 0;
int *__errno_location(void) {
    return &g_errno_val;
}

char *__dummy_environ[] = { (char *)0 };
char **environ = __dummy_environ;

// ----------------------------------------------------------------------------
// Freestanding 64 MB Coalescing Block Memory Allocator for Xenon GDDR3 RAM
// ----------------------------------------------------------------------------
#define XBOX_HEAP_SIZE (64 * 1024 * 1024)
static u8 g_heap[XBOX_HEAP_SIZE] __attribute__((aligned(16)));
static u32 g_heap_top = 0;

struct mem_header {
    u32 magic;      // 0x52434D45 ('RCME')
    u32 size;       // Payload size in bytes
    u32 is_free;    // 1 if free, 0 if in use
    u32 pad;
};

static struct mem_header *g_free_list = (void *)0;

void *malloc(u32 size) {
    if (size == 0) size = 1;
    u32 aligned = (size + 15) & ~15;

    // 1. Search free list for a suitable block
    struct mem_header **prev_next = &g_free_list;
    struct mem_header *curr = g_free_list;
    while (curr) {
        if (curr->magic == 0x52434D45 && curr->is_free && curr->size >= aligned) {
            *prev_next = (struct mem_header *)(*(void **)((u8 *)(curr + 1)));
            curr->is_free = 0;
            return (void *)(curr + 1);
        }
        prev_next = (struct mem_header **)((u8 *)(curr + 1));
        curr = (struct mem_header *)(*prev_next);
    }

    // 2. Allocate from top of heap
    u32 block_total = sizeof(struct mem_header) + aligned;
    if (g_heap_top + block_total > XBOX_HEAP_SIZE) {
        DbgPrint("[RustCraft] Heap exhausted (%u bytes requested, used %u/%u)\n", size, g_heap_top, XBOX_HEAP_SIZE);
        return (void *)0;
    }

    struct mem_header *hdr = (struct mem_header *)&g_heap[g_heap_top];
    g_heap_top += block_total;

    hdr->magic = 0x52434D45;
    hdr->size = aligned;
    hdr->is_free = 0;
    hdr->pad = 0;

    return (void *)(hdr + 1);
}

void free(void *ptr) {
    if (!ptr) return;
    u8 *p = (u8 *)ptr;
    if (p < g_heap || p >= &g_heap[XBOX_HEAP_SIZE]) {
        return;
    }
    struct mem_header *hdr = (struct mem_header *)ptr - 1;
    if (hdr->magic != 0x52434D45 || hdr->is_free) {
        return;
    }
    hdr->is_free = 1;
    *(void **)((u8 *)(hdr + 1)) = (void *)g_free_list;
    g_free_list = hdr;
}

void *calloc(u32 nmemb, u32 size) {
    u32 total = nmemb * size;
    void *ptr = malloc(total);
    if (ptr) memset(ptr, 0, total);
    return ptr;
}

void *realloc(void *ptr, u32 size) {
    if (!ptr) return malloc(size);
    if (size == 0) {
        free(ptr);
        return (void *)0;
    }
    struct mem_header *hdr = (struct mem_header *)ptr - 1;
    if (hdr->magic == 0x52434D45 && hdr->size >= size) {
        return ptr;
    }
    void *new_ptr = malloc(size);
    if (new_ptr) {
        u32 copy_len = (hdr->magic == 0x52434D45) ? (hdr->size < size ? hdr->size : size) : size;
        memcpy(new_ptr, ptr, copy_len);
        free(ptr);
    }
    return new_ptr;
}

int posix_memalign(void **memptr, u32 alignment, u32 size) {
    if (!memptr) return 22; // EINVAL
    *memptr = malloc(size);
    return (*memptr != (void *)0) ? 0 : 12; // ENOMEM
}

u32 xbox_get_heap_used(void) {
    return g_heap_top;
}

u32 xbox_get_heap_total(void) {
    return XBOX_HEAP_SIZE;
}

// ----------------------------------------------------------------------------
// Hardware Timebase & Nanosecond Clock (50 MHz PowerPC Timebase)
// ----------------------------------------------------------------------------
static inline u64 get_xenon_timebase(void) {
    u32 tbu0, tbl, tbu1;
    do {
        __asm__ volatile ("mftbu %0" : "=r"(tbu0));
        __asm__ volatile ("mftb %0" : "=r"(tbl));
        __asm__ volatile ("mftbu %0" : "=r"(tbu1));
    } while (tbu0 != tbu1);
    return (((u64)tbu0) << 32) | ((u64)tbl);
}

typedef struct {
    long tv_sec;
    long tv_nsec;
} timespec_t;

int clock_gettime(int clk_id, timespec_t *tp) {
    if (!tp) return -1;
    u64 tb = get_xenon_timebase();
    // 50,000,000 ticks per second on Xenon
    tp->tv_sec = (long)(tb / 50000000ULL);
    u64 rem = tb % 50000000ULL;
    tp->tv_nsec = (long)(rem * 20ULL);
    return 0;
}

int clock_nanosleep(int clk_id, int flags, const timespec_t *rqtp, timespec_t *rmtp) {
    if (!rqtp) return -1;
    LARGE_INTEGER interval;
    i64 ticks = ((i64)rqtp->tv_sec * 10000000LL) + ((i64)rqtp->tv_nsec / 100LL);
    interval.QuadPart = -ticks;
    KeDelayExecutionThread(0, 0, &interval);
    return 0;
}

// ----------------------------------------------------------------------------
// POSIX File I/O & System Calls
// ----------------------------------------------------------------------------
int write(int fd, const void *buf, u32 count) {
    if ((fd == 1 || fd == 2) && buf && count > 0) {
        char tmp[256];
        const char *src = (const char *)buf;
        u32 remaining = count;
        while (remaining > 0) {
            u32 chunk = remaining < 255 ? remaining : 255;
            memcpy(tmp, src, chunk);
            tmp[chunk] = '\0';
            DbgPrint("%s", tmp);
            src += chunk;
            remaining -= chunk;
        }
    }
    return (int)count;
}

struct iovec {
    void *iov_base;
    u32 iov_len;
};

int writev(int fd, const struct iovec *iov, int iovcnt) {
    if (!iov || iovcnt <= 0) return 0;
    int total = 0;
    for (int i = 0; i < iovcnt; i++) {
        if (iov[i].iov_base && iov[i].iov_len > 0) {
            write(fd, iov[i].iov_base, iov[i].iov_len);
            total += (int)iov[i].iov_len;
        }
    }
    return total;
}
int close(int fd) { return 0; }
int open(const char *path, int flags, ...) { return -1; }
int open64(const char *pathname, int flags, ...) { return -1; }
int openat64(int dirfd, const char *pathname, int flags, ...) { return -1; }
void *opendir(const char *name) { return (void *)0; }
int read(int fd, void *buf, u32 count) { return 0; }
int poll(void *fds, u32 nfds, int timeout) { return 0; }
int munmap(void *addr, u32 length) { return 0; }
int getrandom(void *buf, u32 buflen, unsigned int flags) {
    if (!buf) return -1;
    u8 *p = (u8 *)buf;
    u64 tb = get_xenon_timebase();
    for (u32 i = 0; i < buflen; i++) {
        tb = tb * 6364136223846793005ULL + 1442695040888963407ULL;
        p[i] = (u8)(tb >> 32);
    }
    return (int)buflen;
}

long syscall(long number, ...) {
    if (number == 221 || number == 240) { // sys_futex on 32-bit PowerPC Linux
        return 0; // Success (futex wait/wake no-op in single-threaded runtime)
    }
    if (number == 359) { // SYS_getrandom on PowerPC Linux
        __builtin_va_list args;
        __builtin_va_start(args, number);
        void *buf = __builtin_va_arg(args, void *);
        u32 buflen = __builtin_va_arg(args, u32);
        unsigned int flags = __builtin_va_arg(args, unsigned int);
        __builtin_va_end(args);
        return getrandom(buf, buflen, flags);
    }
    return -1;
}

int chdir(const char *path) { return 0; }
int chmod(const char *path, u32 mode) { return 0; }
int fchmodat(int dirfd, const char *pathname, u32 mode, int flags) { return 0; }
int linkat(int olddirfd, const char *oldpath, int newdirfd, const char *newpath, int flags) { return -1; }
int mkdir(const char *pathname, u32 mode) { return 0; }
int rename(const char *oldpath, const char *newpath) { return 0; }
int renameat(int olddirfd, const char *oldpath, int newdirfd, const char *newpath) { return 0; }
int setenv(const char *name, const char *value, int overwrite) { return 0; }
int symlink(const char *target, const char *linkpath) { return -1; }
int unlinkat(int dirfd, const char *pathname, int flags) { return 0; }

i64 lseek64(int fd, i64 offset, int whence) { return 0; }
int fstat64(int fd, void *buf) { if (buf) memset(buf, 0, 128); return 0; }
int stat64(const char *path, void *buf) { if (buf) memset(buf, 0, 128); return -1; }
int fstatat64(int dirfd, const char *pathname, void *buf, int flags) { if (buf) memset(buf, 0, 128); return -1; }
int lstat64(const char *path, void *buf) { if (buf) memset(buf, 0, 128); return -1; }
int ftruncate64(int fd, i64 length) { return 0; }
int fsync(int fd) { return 0; }
int fdatasync(int fd) { return 0; }
int futimens(int fd, const void *times) { return 0; }
int utimensat(int dirfd, const char *pathname, const void *times, int flags) { return 0; }
int flock(int fd, int operation) { return 0; }
int fcntl(int fd, int cmd, ...) { return 0; }
int pread64(int fd, void *buf, u32 count, i64 offset) { return 0; }
int pwrite64(int fd, const void *buf, u32 count, i64 offset) { return (int)count; }
int preadv(int fd, const void *iov, int iovcnt, i64 offset) { return 0; }
int pwritev(int fd, const void *iov, int iovcnt, i64 offset) { return 0; }
int readv(int fd, const void *iov, int iovcnt) { return 0; }
int splice(int fd_in, void *off_in, int fd_out, void *off_out, u32 len, u32 flags) { return 0; }
int dup(int oldfd) { return oldfd; }
int dup2(int oldfd, int newfd) { return newfd; }
char *getcwd(char *buf, u32 size) { if (buf && size > 5) { memcpy(buf, "game:", 6); return buf; } return (char *)0; }
char *realpath(const char *path, char *resolved_path) {
    if (resolved_path && path) {
        u32 len = strlen(path);
        memcpy(resolved_path, path, len + 1);
        return resolved_path;
    }
    return (char *)0;
}
int readlink(const char *pathname, char *buf, u32 bufsiz) { return -1; }
int unlink(const char *pathname) { return 0; }
int rmdir(const char *pathname) { return 0; }
int closedir(void *dirp) { return 0; }
int dirfd(void *dirp) { return -1; }
void *fdopendir(int fd) { return (void *)0; }
void *readdir64(void *dirp) { return (void *)0; }

// ----------------------------------------------------------------------------
// Networking / Socket Stubs
// ----------------------------------------------------------------------------
int socket(int domain, int type, int protocol) { return -1; }
int socketpair(int domain, int type, int protocol, int sv[2]) { return -1; }
int bind(int sockfd, const void *addr, u32 addrlen) { return -1; }
int listen(int sockfd, int backlog) { return -1; }
int accept4(int sockfd, void *addr, void *addrlen, int flags) { return -1; }
int connect(int sockfd, const void *addr, u32 addrlen) { return -1; }
int getsockopt(int sockfd, int level, int optname, void *optval, void *optlen) { return -1; }
int setsockopt(int sockfd, int level, int optname, const void *optval, u32 optlen) { return -1; }
int getsockname(int sockfd, void *addr, void *addrlen) { return -1; }
int getpeername(int sockfd, void *addr, void *addrlen) { return -1; }
int send(int sockfd, const void *buf, u32 len, int flags) { return -1; }
int sendto(int sockfd, const void *buf, u32 len, int flags, const void *dest_addr, u32 addrlen) { return -1; }
int sendmsg(int sockfd, const void *msg, int flags) { return -1; }
i64 sendfile64(int out_fd, int in_fd, i64 *offset, u32 count) { return -1; }
int recv(int sockfd, void *buf, u32 len, int flags) { return -1; }
int recvfrom(int sockfd, void *buf, u32 len, int flags, void *src_addr, void *addrlen) { return -1; }
int recvmsg(int sockfd, void *msg, int flags) { return -1; }
int shutdown(int sockfd, int how) { return -1; }
int getaddrinfo(const char *node, const char *service, const void *hints, void **res) { return -1; }
void freeaddrinfo(void *res) {}
const char *gai_strerror(int errcode) { return "network not enabled on console"; }
int __res_init(void) { return -1; }
int gethostname(char *name, u32 len) { if (name && len > 4) { memcpy(name, "xbox", 5); return 0; } return -1; }
int ioctl(int fd, unsigned long request, ...) { return -1; }

// ----------------------------------------------------------------------------
// Threading & Synchronization Stubs
// ----------------------------------------------------------------------------
static void *g_tls_values[64] = {0};
static u32 g_tls_key_counter = 0;

int pthread_mutex_lock(void *mutex) { return 0; }
int pthread_mutex_unlock(void *mutex) { return 0; }
int pthread_attr_destroy(void *attr) { return 0; }

int pthread_create(void **thread, const void *attr, void *(*start_routine)(void *), void *arg) { return -1; }
int pthread_join(void *thread, void **retval) { return 0; }
int pthread_detach(void *thread) { return 0; }
void *pthread_self(void) { return (void *)1; }
int pthread_setname_np(void *thread, const char *name) { return 0; }

int pthread_key_create(u32 *key, void (*destructor)(void *)) {
    if (!key) return 22;
    if (g_tls_key_counter >= 64) return 11;
    *key = g_tls_key_counter++;
    return 0;
}
int pthread_key_delete(u32 key) { return 0; }
int pthread_setspecific(u32 key, const void *value) {
    if (key < 64) {
        g_tls_values[key] = (void *)value;
        return 0;
    }
    return 22;
}
void *pthread_getspecific(u32 key) {
    if (key < 64) return g_tls_values[key];
    return (void *)0;
}

typedef struct {
    unsigned long ti_module;
    unsigned long ti_offset;
} tls_index;

void *__tls_get_addr(tls_index *ti) {
    static char g_static_tls_block[8192];
    if (ti) {
        return (void *)(g_static_tls_block + (ti->ti_offset % 8192));
    }
    return (void *)g_static_tls_block;
}

int pthread_attr_init(void *attr) { return 0; }
int pthread_attr_getstack(const void *attr, void **stackaddr, u32 *stacksize) {
    if (stackaddr) *stackaddr = (void *)0x80000000;
    if (stacksize) *stacksize = 1024 * 1024;
    return 0;
}
int pthread_attr_getguardsize(const void *attr, u32 *guardsize) { if (guardsize) *guardsize = 4096; return 0; }
int pthread_attr_setstacksize(void *attr, u32 stacksize) { return 0; }
int pthread_getattr_np(void *thread, void *attr) { return 0; }
int sched_yield(void) { return 0; }
int sched_getaffinity(int pid, u32 cpusetsize, void *mask) {
    if (mask && cpusetsize >= 4) {
        *(u32 *)mask = 0x01; // 1 hardware thread (single-threaded for stability)
        return 0;
    }
    return -1;
}

// ----------------------------------------------------------------------------
// Process, Signals & Abort
// ----------------------------------------------------------------------------
void abort(void) {
    DbgPrint("[RustCraft] abort() called from game runtime!\n");
    HalReturnToFirmware(2);
    ExTerminateThread(1);
    while (1) {}
}
void exit(int status) {
    DbgPrint("[RustCraft] exit(%d) called\n", status);
    HalReturnToFirmware(2);
    ExTerminateThread((u32)status);
    while (1) {}
}
void _exit(int status) { exit(status); }

int fork(void) { return -1; }
int execvp(const char *file, char *const argv[]) { return -1; }
int waitpid(int pid, int *wstatus, int options) { return -1; }
int waitid(int idtype, int id, void *infop, int options) { return -1; }
int kill(int pid, int sig) { return -1; }
int killpg(int pgrp, int sig) { return -1; }
void (*signal(int signum, void (*handler)(int)))(int) { return (void (*)(int))0; }
int sigaction(int signum, const void *act, void *oldact) { return 0; }
int sigemptyset(void *set) { if (set) memset(set, 0, 128); return 0; }
int sigaddset(void *set, int signum) { return 0; }
int sigaltstack(const void *ss, void *old_ss) { return 0; }
int pause(void) { return 0; }
int pipe2(int pipefd[2], int flags) { return -1; }
int posix_spawnp(int *pid, const char *file, const void *file_actions, const void *attrp, char *const argv[], char *const envp[]) { return -1; }
int posix_spawn_file_actions_init(void *actions) { return 0; }
int posix_spawn_file_actions_destroy(void *actions) { return 0; }
int posix_spawn_file_actions_adddup2(void *actions, int fd, int newfd) { return 0; }
int posix_spawnattr_init(void *attr) { return 0; }
int posix_spawnattr_destroy(void *attr) { return 0; }
int posix_spawnattr_setflags(void *attr, short flags) { return 0; }
int posix_spawnattr_setpgroup(void *attr, int pgroup) { return 0; }
int posix_spawnattr_setsigdefault(void *attr, const void *sigdefault) { return 0; }

// ----------------------------------------------------------------------------
// Environment & Misc
// ----------------------------------------------------------------------------
char *getenv(const char *name) { return (char *)0; }
int unsetenv(const char *name) { return 0; }
long sysconf(int name) {
    if (name == 30) return 4096; // _SC_PAGESIZE
    if (name == 84) return 6;    // _SC_NPROCESSORS_ONLN (Xenon 3 cores x 2 SMT)
    return -1;
}
unsigned long getauxval(unsigned long type) { return 0; }
int getpid(void) { return 1; }
int getppid(void) { return 1; }
int getuid(void) { return 0; }
int getpwuid_r(int uid, void *pwd, char *buf, u32 buflen, void **result) { if (result) *result = (void *)0; return -1; }
int setuid(int uid) { return 0; }
int setgid(int gid) { return 0; }
int setgroups(u32 size, const void *list) { return 0; }
int setpgid(int pid, int pgid) { return 0; }
int setsid(void) { return 1; }
int chown(const char *pathname, int owner, int group) { return 0; }
int lchown(const char *pathname, int owner, int group) { return 0; }
int fchown(int fd, int owner, int group) { return 0; }
int fchmod(int fd, u32 mode) { return 0; }
int chroot(const char *path) { return -1; }
int mkfifo(const char *pathname, u32 mode) { return -1; }
const char *gnu_get_libc_version(void) { return "2.36"; }
int __xpg_strerror_r(int errnum, char *buf, u32 buflen) {
    if (buf && buflen > 5) { memcpy(buf, "error", 6); }
    return 0;
}
u32 strnlen(const char *s, u32 maxlen) {
    u32 i = 0;
    while (i < maxlen && s[i]) i++;
    return i;
}
void *dlsym(void *handle, const char *symbol) { return (void *)0; }
void *mmap64(void *addr, u32 length, int prot, int flags, int fd, i64 offset) {
    return malloc(length);
}
int mprotect(void *addr, u32 len, int prot) { return 0; }

// ----------------------------------------------------------------------------
// DWARF Exception Unwinding Stubs
// ----------------------------------------------------------------------------
int dl_iterate_phdr(void *callback, void *data) { return 0; }

void _Unwind_Resume(void *exception) {
    while (1) {}
}

void *_Unwind_FindEnclosingFunction(void *pc) { return pc; }
u32 _Unwind_GetCFA(void *context) { return 0; }
u32 _Unwind_GetIP(void *context) { return 0; }
u32 _Unwind_GetTextRelBase(void *context) { return 0; }
int _Unwind_Backtrace(void *trace, void *trace_argument) { return 0; }
u32 _Unwind_GetDataRelBase(void *context) { return 0; }
u32 _Unwind_GetIPInfo(void *context, int *ip_before_insn) { if (ip_before_insn) *ip_before_insn = 0; return 0; }
void *_Unwind_GetLanguageSpecificData(void *context) { return (void *)0; }
u32 _Unwind_GetRegionStart(void *context) { return 0; }
void _Unwind_SetGR(void *context, int index, u32 val) {}
void _Unwind_SetIP(void *context, u32 val) {}

// ============================================================================
// 4. Xenon Hardware Video (Xenos Framebuffer) & Controller Subsystem
// ============================================================================
struct ati_info {
    u32 unknown1[4];
    u32 base;        // 0x6110 (D1GRPH_PRIMARY_SURFACE_ADDRESS)
    u32 unknown2[8];
    u32 width;       // 0x6134 (D1GRPH_X_END)
    u32 height;      // 0x6138 (D1GRPH_Y_END)
} __attribute__((packed));

static void *g_xenon_vram = (void *)0;
static int g_xenon_fb_w = 1280;
static int g_xenon_fb_h = 720;

// In-memory simulated framebuffer for running safely under xboxkrnl.exe
static u32 g_simulated_fb[1280 * 720] __attribute__((aligned(16)));

int is_running_under_kernel_c(void) {
    volatile u32 *thunk = (volatile u32 *)DbgPrint;
    if (*thunk != 0x01000003) {
        return 1;
    }
    extern u32 imp_desc_DbgPrint;
    if (imp_desc_DbgPrint != 0x00000003 && (imp_desc_DbgPrint & 0x80000000)) {
        return 1;
    }
    return 0;
}

static inline int is_running_under_kernel(void) {
    return is_running_under_kernel_c();
}

void xenon_uart_puts(const char *s) {
    if (!s) return;
    if (is_running_under_kernel()) {
        DbgPrint("%s", s);
        return;
    }
    // Bare-metal Xenon UART TX (FIFO status at 0xea001018, TX data at 0xea001014)
    volatile u32 *uart_status = (volatile u32 *)0xea001018;
    volatile u32 *uart_data   = (volatile u32 *)0xea001014;
    while (*s) {
        while (*uart_status & 0x02) {}
        *uart_data = (u32)(*s++);
    }
}

void *video_get_vram(void) {
    if (is_running_under_kernel()) {
        return g_simulated_fb;
    }
    if (g_xenon_vram) return g_xenon_vram;

    // Check Xenos display registers at 0xec806100 (ATI info) only under XeLL!
    volatile struct ati_info *ai = (volatile struct ati_info *)0xec806100;
    u32 base = ai->base;
    u32 w = ai->width;
    u32 h = ai->height;

    // In XeLL / bare-metal mode, 0xec806110 contains the active display physical surface base
    if (base != 0 && base != 0xFFFFFFFF && w >= 640 && h >= 480 && w <= 1920 && h <= 1080) {
        g_xenon_fb_w = w;
        g_xenon_fb_h = h;
        // Map with 0xC0000000 (uncached, writes directly to scanout RAM without cache delay)
        g_xenon_vram = (void *)(base | 0xc0000000);
        return g_xenon_vram;
    }

    // Standard XeLL fallback: 0x1E000000 -> uncached 0xDE000000
    g_xenon_fb_w = 1280;
    g_xenon_fb_h = 720;
    g_xenon_vram = (void *)0xde000000;
    return g_xenon_vram;
}

void video_init(void) {
    video_get_vram();
}

void video_set_mode(int mode) {
    (void)mode;
}

void xenon_present_framebuffer(void *vram, const u32 *pixels, int src_w, int src_h, int dst_w, int dst_h) {
    if (!pixels) return;
    if (is_running_under_kernel()) {
        if (!vram) vram = g_simulated_fb;
        int copy_w = (dst_w < src_w) ? dst_w : src_w;
        int copy_h = (dst_h < src_h) ? dst_h : src_h;
        u32 *dst = (u32 *)vram;
        for (int y = 0; y < copy_h; y++) {
            memcpy(&dst[y * dst_w], &pixels[y * src_w], copy_w * sizeof(u32));
        }
        return;
    }

    if (!vram) {
        vram = video_get_vram();
    }
    if (!vram) return;

    u32 *fb_uncached = (u32 *)vram;
    u32 *fb_cached   = (u32 *)0x9e000000;

    int copy_w = (dst_w < src_w) ? dst_w : src_w;
    int copy_h = (dst_h < src_h) ? dst_h : src_h;

    // Direct linear copy to uncached physical memory for immediate GPU scanout
    for (int y = 0; y < copy_h; y++) {
        const u32 *src_row = &pixels[y * src_w];
        u32 *dst_row = &fb_uncached[y * dst_w];
        for (int x = 0; x < copy_w; x++) {
            dst_row[x] = src_row[x];
        }
    }

    // Also mirror to cached segment and flush CPU L2 cache lines only in XeLL
    u8 *p_cached = (u8 *)fb_cached;
    u32 total_bytes = dst_w * dst_h * 4;
    for (u32 i = 0; i < total_bytes; i += 128) {
        __asm__ volatile ("dcbst 0, %0" : : "r"(p_cached + i));
    }
    __asm__ volatile ("sync");
}

// Packed 26-byte structure exactly matching Rust's ControllerData
struct ControllerData {
    u8 a;
    u8 b;
    u8 x;
    u8 y;
    u8 up;
    u8 down;
    u8 left;
    u8 right;
    u8 start;
    u8 back;
    u8 xenon; // Guide button
    u8 lb;
    u8 rb;
    u8 lt;
    u8 rt;
    short s1_x;
    short s1_y;
    short s2_x;
    short s2_y;
    u8 s1_z;
    u8 s2_z;
    u8 logo;
} __attribute__((packed));

void usb_do_poll(void) {}

int get_controller_data(struct ControllerData *data, int port) {
    if (!data) return 0;
    u8 *p = (u8 *)data;
    for (int i = 0; i < sizeof(struct ControllerData); i++) {
        p[i] = 0;
    }
    return 1;
}

void xenon_smc_power_shutdown(void) {
    if (is_running_under_kernel()) {
        HalReturnToFirmware(1);
    } else {
        while (1) {}
    }
}

void xenon_smc_power_reboot(void) {
    if (is_running_under_kernel()) {
        HalReturnToFirmware(2);
    } else {
        while (1) {}
    }
}

// ============================================================================
// 5. File I/O Diagnostic Logging Subsystem
// ============================================================================
#define LOG_ACCUM_SIZE (64 * 1024)
static char g_accum_log[LOG_ACCUM_SIZE];
static u32 g_accum_log_len = 0;
static char g_bound_log_path[128] = {0};

static int raw_write_nt_file(const char *nt_path, void *root_dir, const void *buf, u32 len) {
    if (!is_running_under_kernel()) return -1;

    ANSI_STRING ansi_path;
    ansi_path.Length = (u16)strlen(nt_path);
    ansi_path.MaximumLength = ansi_path.Length + 1;
    ansi_path.Buffer = (char *)nt_path;

    OBJECT_ATTRIBUTES obj_attrs;
    obj_attrs.RootDirectory = root_dir;
    obj_attrs.ObjectName = &ansi_path;
    obj_attrs.Attributes = 0x00000040; // OBJ_CASE_INSENSITIVE

    IO_STATUS_BLOCK io_status;
    void *file_handle = (void *)0;

    u32 desired_access = 0x40100000; // GENERIC_WRITE | SYNCHRONIZE
    u32 file_attrs = 0x00000080;     // FILE_ATTRIBUTE_NORMAL
    u32 share_access = 0x00000003;   // FILE_SHARE_READ | FILE_SHARE_WRITE
    u32 create_disp = 0x00000005;    // FILE_OVERWRITE_IF
    u32 create_opts = 0x00000060;    // FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE

    u32 status = NtCreateFile(
        &file_handle,
        desired_access,
        &obj_attrs,
        &io_status,
        (void *)0,
        file_attrs,
        share_access,
        create_disp,
        create_opts
    );

    if (status == 0 && file_handle != (void *)0) {
        IO_STATUS_BLOCK write_status;
        NtWriteFile(
            file_handle,
            (void *)0,
            (void *)0,
            (void *)0,
            &write_status,
            buf,
            len,
            (void *)0
        );
        NtClose(file_handle);
        return 0; // Success!
    }

    return (int)status;
}

void xbox_flush_log_to_disk(void) {
    if (!is_running_under_kernel() || g_accum_log_len == 0) {
        return;
    }

    // If already bound to a verified working path, write directly
    if (g_bound_log_path[0] != '\0') {
        void *root = (g_bound_log_path[0] == '\\') ? (void *)0 : (void *)0xFFFFFFFD;
        raw_write_nt_file(g_bound_log_path, root, g_accum_log, g_accum_log_len);
        return;
    }

    // Try candidate paths in order of preference
    struct {
        const char *path;
        void *root;
    } candidates[] = {
        { "\\??\\game:\\rustcraft.log", (void *)0 },
        { "game:\\rustcraft.log", (void *)0xFFFFFFFD },
        { "\\??\\Usb0:\\rustcraft.log", (void *)0 },
        { "Usb0:\\rustcraft.log", (void *)0xFFFFFFFD },
        { "\\??\\Usb0:\\RustCraft\\rustcraft.log", (void *)0 },
        { "Usb0:\\RustCraft\\rustcraft.log", (void *)0xFFFFFFFD },
        { "\\Device\\Harddisk0\\Partition1\\rustcraft.log", (void *)0 },
        { "\\Device\\Harddisk0\\Partition1\\RustCraft\\rustcraft.log", (void *)0 },
        { "\\Device\\Mass0Partition1\\rustcraft.log", (void *)0 },
        { "\\Device\\Mass0Partition1\\RustCraft\\rustcraft.log", (void *)0 },
        { "\\Device\\Mass1Partition1\\rustcraft.log", (void *)0 },
        { "\\Device\\Mass1Partition1\\RustCraft\\rustcraft.log", (void *)0 },
        { "\\Device\\Mass0\\rustcraft.log", (void *)0 },
        { "\\Device\\Mass0\\RustCraft\\rustcraft.log", (void *)0 },
    };

    int num_candidates = sizeof(candidates) / sizeof(candidates[0]);
    for (int i = 0; i < num_candidates; i++) {
        int res = raw_write_nt_file(candidates[i].path, candidates[i].root, g_accum_log, g_accum_log_len);
        if (res == 0) {
            u32 plen = strlen(candidates[i].path);
            if (plen < sizeof(g_bound_log_path)) {
                memcpy(g_bound_log_path, candidates[i].path, plen + 1);
            }
            DbgPrint("[RustCraft] ✓ Log file successfully created at: %s\n", candidates[i].path);
            return;
        }
    }
}

void xbox_append_log_c(const char *msg, u32 len) {
    if (!msg || len == 0) return;

    // Send formatted message to DbgPrint
    DbgPrint("[XBOX360] %.*s\n", (int)len, msg);

    // Send to UART if in XeLL
    if (!is_running_under_kernel()) {
        xenon_uart_puts(msg);
        xenon_uart_puts("\n");
    }

    // Append to accumulator buffer
    if (g_accum_log_len + len + 2 < LOG_ACCUM_SIZE) {
        memcpy(&g_accum_log[g_accum_log_len], msg, len);
        g_accum_log_len += len;
        g_accum_log[g_accum_log_len++] = '\r';
        g_accum_log[g_accum_log_len++] = '\n';
        g_accum_log[g_accum_log_len] = '\0';
    }

    // Immediate flush to disk under kernel
    xbox_flush_log_to_disk();
}

const char *xbox_get_bound_log_path(void) {
    return g_bound_log_path[0] ? g_bound_log_path : "Pending (USB / HDD)";
}

// ============================================================================
// 6. Primary Entry Point at 0x82001000
// ============================================================================
__attribute__((section(".text.entry"), used))
void _start(void) {
    int under_kernel = is_running_under_kernel();

    if (!under_kernel) {
        // XeLL Reloaded: 0xDE000000 is safe! Clear screen to dark slate
        volatile u32 *fb = (volatile u32 *)0xde000000;
        for (int i = 0; i < 1280 * 720; i++) {
            fb[i] = 0xFF0F172A; // Slate 900
        }
        u8 *p_cached = (u8 *)0x9e000000;
        for (u32 i = 0; i < 1280 * 720 * 4; i += 128) {
            __asm__ volatile ("dcbst 0, %0" : : "r"(p_cached + i));
        }
        __asm__ volatile ("sync");
    }

    const char *b1 = "============================================================";
    const char *b2 = "RustCraft Microsoft Xbox 360 Native Launcher v1.2";
    xbox_append_log_c(b1, strlen(b1));
    xbox_append_log_c(b2, strlen(b2));
    if (under_kernel) {
        const char *m1 = "[Environment] Running under xboxkrnl.exe (Dashboard / XEX Title)";
        const char *m2 = "[Hardware] Xenon 3.2 GHz PowerPC Tri-Core active (xboxkrnl.exe NT Container)";
        xbox_append_log_c(m1, strlen(m1));
        xbox_append_log_c(m2, strlen(m2));
    } else {
        const char *m1 = "[Environment] Running under XeLL Reloaded (Bare-Metal)";
        const char *m2 = "[Hardware] Xenon 3.2 GHz PowerPC Tri-Core active (Direct VRAM 0xDE000000)";
        xbox_append_log_c(m1, strlen(m1));
        xbox_append_log_c(m2, strlen(m2));
    }
    xbox_append_log_c(b1, strlen(b1));

    // Launch the full RustCraft game engine compiled in Rust
    extern void run_rustcraft_xbox360(void);
    run_rustcraft_xbox360();

    if (under_kernel) {
        const char *end_msg = "[RustCraft] Game engine execution completed. Returning to dashboard.";
        xbox_append_log_c(end_msg, strlen(end_msg));
        xbox_flush_log_to_disk();
        HalReturnToFirmware(2);
        ExTerminateThread(0);
    } else {
        const char *halt_msg = "[RustCraft] Game engine finished. Halting.";
        xbox_append_log_c(halt_msg, strlen(halt_msg));
        while (1) {}
    }
}


