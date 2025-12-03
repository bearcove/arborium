#define _GNU_SOURCE

#include <stddef.h>
#include <stdbool.h>
#include <immintrin.h>
#include <x86intrin.h>
#include <string.h>
#include <unistd.h>
#include <stdint.h>
#include <fcntl.h>
#include <sys/uio.h>
#include <stdio.h>
#include <errno.h>
#include <err.h>
#include <limits.h>
#include <unistd.h>
#include <sys/mman.h>

// #define NO_PRINT

#define max(a,b)             \
({                           \
    __typeof__ (a) _a = (a); \
    __typeof__ (b) _b = (b); \
    _a > _b ? _a : _b;       \
})

#define min(a,b)             \
({                           \
    __typeof__ (a) _a = (a); \
    __typeof__ (b) _b = (b); \
    _a < _b ? _a : _b;       \
})

// number of parenthesis pairs
#ifndef SIZE
#define SIZE 20
#endif

// number of parenthesis characters in output
#define PSIZE (SIZE * 2)

// bytes per line - one more fore LF
#define LSIZE (PSIZE + 1)

// bitmask of parenthesis
#define PMASK ((1ULL << PSIZE) - 1)

// End state
#define FIN (((1ULL << SIZE) - 1) << (SIZE))

// upper size of the buffer, actual size will be smaller
// right now this is as big as it can be
#define BUFSIZE ((1 << 10) << 11)
#define DUAL_BUFSIZE (2 * BUFSIZE)
#define BUFALIGN (1 << 22)
#define BUFSPLIT (BUFALIGN >> 1)

// I am consistantly surprised at how big it's practical for the cache size to
// be. Doing tperf tests as of v6.0 with 1<<19 (512kiB) gives cache miss rate of
// 0.009% but 1<<20 (1MiB) has only 0.011% cache misses. It's not enough to make
// the smaller cache faster (actually it's slower by ~100MiB)
//
// My L2 cache is 256 kiB per CPU and my L3 is 12 MiB
#define CACHESIZE (1l << 18)

// SIMD batch size (in bytes) such that AVX2 stores can be used
// additional byte per line for LF
// this is the size of the SIMD REGISTER
#define SIMD_BYTES 32

// #if (LSIZE < 32)
// #error "need code for shorter lines"
// #endif
#if ((LSIZE & 1) == 0)
#error "LSIZE should be odd how tf did this happen"
#endif

// number of store operations in a batch
#define BATCH_STORE (LSIZE)

#if SIZE < 1 || SIZE >= 32
#error "unsupported size"
#endif

/* 
 * pairs and their cycles ( ()()()() -> (((()))) would be 4 pairs)
 * 6 pairs - period 132
 * 5 pairs - period 42
 * 4 pairs - period 14
 * 3 pairs - period 5
 * 2 pairs - period 2
 */


// number of bytes in a series of vector stores such that LF is at the end
// Since BATCH_STORE is odd and SIMD_BYTES a power of 2, the lcm is their
// product
#define BATCH_BYTES (BATCH_STORE * SIMD_BYTES)

// lines in a batch
#define BATCH_LINES (BATCH_BYTES / LSIZE)


// number of batch writes that fit in a pipe
#define PIPECNT (((CACHESIZE) / SIMD_BYTES))


// https://stackoverflow.com/a/1898487
#define is_aligned(ptr, align) \
    (((uintptr_t)(const void *)(ptr)) % (align) == 0)

#ifdef DEBUG
#define ERROR(msg) err(1, "%s", msg)
#else 
#define ERROR(msg) exit_fail()
// #define ERROR(msg) err(1, "%s", msg)
#endif

#define unlikely(x) __builtin_expect((x), 0)

#define STR_(x) #x
#define STR(x) STR_(x)


// io buffers need to be 32-byte alligned because otherwise 
// _mm256_store_si256 generates a general protection fault
static char __attribute__ ((aligned(BUFALIGN))) buf[DUAL_BUFSIZE];

// bytecode here is just doing as much pre-computation as possible. All the hot
// loop needs to do is distribute bits to low pos, then add with bytecode
static char __attribute__ ((aligned(32))) bytecode[BATCH_BYTES];


// #if (PSIZE < 32)
// #error "bytecode and batch cannot handle smaller than 32"
// #endif

static void
print_bits(uint64_t bits)
{
	char arr[65];
	for (int i = 0; i < 64; i++) {
		arr[i] = ((bits >> i) & 1) + '0';
	}
	arr[64] = '\0';
	puts(arr);
}

static void
mm256_print_bytes(__m256i bytes)
{
	unsigned char arr[32];
	_mm256_storeu_si256((__m256i *)arr, bytes);
	printf("[");
	for (int i = 0; i < 32; i++) {
		printf("%2x ", (int)arr[i]);
	}
	printf("]\n");
}

static void
mm_print_bytes(__m128i bytes)
{
	unsigned char arr[16];
	_mm_storeu_si128((__m128i *)arr, bytes);
	printf("[");
	for (int i = 0; i < 16; i++) {
		printf("%2x ", (int)arr[i]);
	}
	printf("]\n");
}

__attribute__((cold)) static void
gen_bytecode(uint64_t paren)
{
	int i;

	(void)paren;

	i = 0;
	for (; i < BATCH_BYTES; i++) {
		if ((i + 1) % LSIZE == 0) {
			bytecode[i] = '\n';
		} else if ((i + 0) % LSIZE >= 32) {
			// the upper bits are changed less often, so we can
			// precompute one for a few tens of thousands of lines
			// bytecode[i] = 0x28 + ((paren >> (i % LSIZE)) & 0x1);
			// bytecode[i] = '0';
			bytecode[i] = 0x28;
		} else {
			bytecode[i] = 0x28;
		}
	}
}

__attribute__((cold, noreturn)) static void 
exit_fail(void) 
{
	dprintf(STDERR_FILENO, "error occured.\n");
	exit(2);
}

[[gnu::noinline]]
static char *
flush_buf(size_t bcnt, char *currbuf)
{
	#ifdef NO_PRINT
	return buf + ((currbuf - buf) ^ BUFSPLIT);
	#endif

	ssize_t rem, amt;

	struct iovec iov = {
		.iov_base = currbuf,
		.iov_len = 0,
	};


	// using vmsplice to reduce write(2) overhead
	rem = bcnt;
	amt = 0;
	while(rem > 0) {
		iov.iov_len = rem;
		iov.iov_base += amt;

		#ifndef DEBUG
			amt = vmsplice(STDOUT_FILENO, &iov, 1, 0);
		#else
			amt = write(STDOUT_FILENO, iov.iov_base, iov.iov_len);
		#endif
		if (unlikely(amt == -1)) {
			ERROR("writing error");
		}
		rem -= amt;
	}

	// swap out other buffer
	// we do this to be sure the previous pipe is drained
	return buf + ((currbuf - buf) ^ BUFSPLIT);
}


/*
 * general strategy:
 *
 * curr is a bitmask of all the close parentheses with LSb the start of
 * the output string. 
 *
 * We find the rightmost contiguous bit, and reset all but the MSb of the
 * contiguous group that it's in to the original position. We then take that
 * remaining bit and swap it with the next most significant bit
 *
 * This is optimized by adding a 1 that's shifted to start of group, which is
 * effectively a swap and clear simultaniously
 *
 * Here's a simple example of the function in action, remember the least
 * significant bit is the rightmost in the output:
 *
 * curr = 1010111000 = ((()))()()
 *
 *               1010111000
 * first = 3           ^---
 *
 *               1010111000
 * contig = 3        ^^^
 *
 *               1010111000
 * add:        +       1000
 *             = 1011000000 = (((((())()
 *
 * move contig - 1 = 2 bits back to their original positions
 *
 * rst:        = 0000001010 = ()()((((((
 *
 * return add + rst
 *             = 1011001010 = ()()(())()
 *
 */
[[gnu::always_inline]]
static inline uint64_t
next_paren_bitmask(uint64_t curr)
{
	// first set bit
	const uint64_t first = _tzcnt_u64(curr);

	// carry forward
	const uint64_t add = curr + _blsi_u64(curr);

	// number of contig bits grouped with first
	// const uint64_t contig = _tzcnt_u64(add) - first;

	// original bit positions
	const uint64_t orig = 0xAAAAAAAAAAAAAAAA; // 0b1010...

	// the bits that are moved back to their original position.
	// I've found quite a bit of speedup through these iterations
	// const uint64_t rst = _pdep_u64((1 << (contig - 1)) - 1, orig);
	// const uint64_t rst = orig & ((1 << (contig * 2 - 1)) - 1);
	// const uint64_t rst = orig & ((1 << ((contig << 1) - 1)) - 1);
	// const uint64_t rst = _bextr_u64(orig, 0, contig * 2 - 2);
	// const uint64_t rst = _bzhi_u64(orig, contig * 2 - 1);
	// const uint64_t rst = _pdep_u64((_blsi_u64(add) >> (first + 1)) - 1, orig);
	// const uint64_t rst = _bzhi_u64(orig << 1, contig * 2) >> 1;
	// const uint64_t rst = _pdep_u64(_blsmsk_u64(add >> 2) >> first, orig);
	// const uint64_t rst = _pdep_u64(_blsmsk_u64(add) >> first, orig) >> 4;
	const uint64_t rst = _pdep_u64(_blsmsk_u64(add) >> (first + 2), orig);

	const uint64_t ret = add | rst;

	// need new bytcode if change in upper dword
	// This is currently incorrect, we need to do keep the bytecode
	// consistent across split acesses
	// if (unlikely((0xFFFFFFFF00000000 & add) > curr)) {
	// 	gen_bytecode(ret);
	// }

	return ret;
}

static void 
do_batch(uint64_t paren)
{
	if (PSIZE < 32) {
		ERROR("batch is for lines longer than 32");
	}

	int i;
	uint64_t bcidx;
	int64_t voff;
	__m256i resv, bcv;
	uint64_t curr;
	char *cursor, *currbuf;

	const __m256i shufmask = _mm256_set_epi64x(
					0x0303030303030303,
					0x0202020202020202,
					0x0101010101010101,
					0x0000000000000000);
	const __m256i andmask = _mm256_set1_epi64x(0x8040201008040201);

	cursor = buf;
	currbuf = buf;

	voff = LSIZE;
	i = 0;
	bcidx = 0;
	curr = (uint32_t)paren;
	do {
		// curr = paren >> poff;
		if (voff < 32) {
			paren = next_paren_bitmask(paren);
			curr |= (uint64_t)(uint32_t)paren << voff;
			voff += LSIZE;
		} 
		voff -= 32;
		// print_bits(curr);

		// also try _mm256_lddqu_si256
		bcv = _mm256_load_si256((__m256i *)&bytecode[bcidx]);
		bcidx += SIMD_BYTES;
		if (bcidx == BATCH_BYTES) {
			bcidx = 0;
		}

		// trying to find a 256-bit deposit equivalent
		// if we move each byte of the 32-bit paren to the qword it
		// belongs to, we can just AND it with that bit set

		// only need the low 32 bits of each lane set, but this is fine
		resv = _mm256_set1_epi32(curr);

		// move the byte of paren that has the bit in the corresponding
		// position in the vector to that position.
		resv = _mm256_shuffle_epi8(resv, shufmask);

		// only let the correct bit be set
		resv = _mm256_and_si256(resv, andmask);

		// set all nonzero bytes to -1
		// reuse andmask because it's a superset of resv
		resv = _mm256_cmpeq_epi8(resv, andmask);

		// combine with bytecode
		resv = _mm256_sub_epi8(bcv, resv);


		_mm256_store_si256((__m256i *) cursor, resv);
		cursor += 32;

		curr >>= 32;


		// for whatever reason, transfers are done 128 bytes less
		// than expected (check with strace)
		//
		// for whatever reason, keeping the `paren == FIN` decreases
		// brach mispredictions by ~3%. Combined with removing the
		// post-loop flush, it decreases by 95%. Only doing the latter
		// brings no benefit. Why? let me grab my grimoire so I can
		// summon the Great Old Ones in search of answers.
		if (i >= PIPECNT || paren == FIN) {
			currbuf = flush_buf(cursor - currbuf, currbuf);
			cursor = currbuf;
			i = 0;
		}
		i += 1;
	} while(paren != FIN);
	// flush_buf((i) * BATCH_BYTES);
}

static void 
do_batch_unrolled_native(uint64_t paren)
{
	int i, j;
	int voff, prev_voff;
	__m256i resv, bcv;
	uint64_t curr;
	char *cursor, *currbuf;

	const __m256i shufmask = _mm256_set_epi64x(
					0x0303030303030303,
					0x0202020202020202,
					0x0101010101010101,
					0x0000000000000000);
	const __m256i andmask = _mm256_set1_epi64x(0x8040201008040201);

	cursor = buf;
	currbuf = buf;

	i = 0;
	curr = paren;
	do {
		voff = LSIZE;
		prev_voff = 0;

		#pragma GCC unroll 9999
		for (j = 0; j < LSIZE; j++) {
			if (prev_voff + LSIZE > 64 && prev_voff < 32) {
				curr |= paren >> (32 - prev_voff);
			}

			prev_voff = voff;

			while (voff < 32) {
				prev_voff = voff;
				paren = next_paren_bitmask(paren);
				curr |= paren << voff;
				voff += LSIZE;
			} 
			voff -= 32;

			bcv = _mm256_load_si256((__m256i *)&bytecode[j * 32]);

			// trying to find a 256-bit deposit equivalent if we
			// move each byte of the 32-bit paren to the qword it
			// belongs to, we can just AND it with that bit set

			// only need the low 32 bits of each lane set, but this
			// is fine
			resv = _mm256_set1_epi32(curr);

			// move the byte of paren that has the bit in the
			// corresponding position in the vector to that
			// position.
			resv = _mm256_shuffle_epi8(resv, shufmask);

			// only let the correct bit be set
			resv = _mm256_and_si256(resv, andmask);

			// set all nonzero bytes to -1
			// reuse andmask because it's a superset of resv
			resv = _mm256_cmpeq_epi8(resv, andmask);

			// combine with bytecode
			resv = _mm256_sub_epi8(bcv, resv);


			_mm256_store_si256((__m256i *) cursor, resv);
			cursor += 32;

			curr >>= 32;

			if (i >= PIPECNT || paren == FIN) {
				currbuf = flush_buf(cursor - currbuf, currbuf);
				cursor = currbuf;
				i = 0;
				if (paren == FIN) {
					return;
				}
			}
			i += 1;
		}

		paren = next_paren_bitmask(paren);
		curr = paren;
	} while(paren != FIN);
}

#include "unrolled.generated.h"

static void
flat_store(uint64_t paren)
{
	(void)paren;
	/* 
	 * this is a super minimal experiment of how fast I could expect to get
	 * in a single-threaded context. This function is just stores
	 *
	 * This function gets 26.7 GiB/s 
	 * Jumps to 38 GiB/s with 128kb CACHESIZE
	 */

	/*
	int i;
	char *lc;
	const __m256i batch = _mm256_set1_epi64x(0xFF00FF00FF00FF00);

	i = 0;
	lc = currbuf;
	while(true) {
		_mm256_store_si256((__m256i *) lc, batch);
		lc += 32;

		if (i >= PIPECNT) {
			// flush_buf((i) * SIMD_BYTES);
			flush_buf(lc - currbuf);
			lc = currbuf;
			i = 0;
		}
		i += 1;
	};
	*/
}

static void
flat_flush_buf(uint64_t paren)
{
	(void)paren;
	/* 
	 * this is a super minimal experiment of how fast I could expect to get
	 * in a single-threaded context. This function is just the speed of
	 * flush_buf
	 *
	 * This function gets 72.8 GiB/s 
	 */

	/*
	memset(buf, 0xFF, DUAL_BUFSIZE);
	while(true) {
		flush_buf(CACHESIZE);
	};
	*/
}


void 
_start(void)
{
	uint64_t paren;

	if ( !is_aligned(buf, BUFALIGN)) {
		ERROR("buf is not aligned");
	}

	if (madvise(buf, DUAL_BUFSIZE, MADV_HUGEPAGE) == -1) {
		ERROR("huge page fail");
	}

	#ifndef DEBUG
	if (fcntl(STDOUT_FILENO, F_SETPIPE_SZ, CACHESIZE) != CACHESIZE) {
		ERROR("Pipe could not be resized");
	}
	#endif

	// 9 at the end to initialize into correct place
	// (removed for batching)
	paren = PMASK & 0xAAAAAAAAAAAAAAAA;
	gen_bytecode(paren);
	// do_batch(paren);
	// do_batch_unrolled(paren);
	do_batch_unrolled_native(paren);
	// flat_store_bytecode(paren);
	// flat_store(paren);
	// flat_flush_buf(paren);
	close(STDOUT_FILENO);
	exit(0);
}

/*
vim: tw=80 sw=8
*/
