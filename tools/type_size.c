#include <signal.h>
#include <stdlib.h>
#include <stdio.h>
#include <sys/time.h>
#include <sys/types.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <sys/stat.h>

int main() {
	printf("struct sigaction=%d\n", sizeof(struct sigaction));
	printf("sigset_t=%d\n", sizeof(sigset_t));
	printf("long=%d\n", sizeof(long));
	printf("time_t=%d\n", sizeof(time_t));
	printf("struct timeval=%d\n", sizeof(struct timeval));
	printf("suseconds_t=%d\n", sizeof(suseconds_t));
	printf("rusage=%d\n", sizeof(struct rusage));
	printf("struct stat=%d\n", sizeof(struct stat));
	printf("dev_t=%d\n", sizeof(dev_t));
	printf("ino_t=%d\n", sizeof(ino_t));
	printf("mode_t=%d\n", sizeof(mode_t));
	printf("nlink_t=%d\n", sizeof(nlink_t));
	printf("uid_t=%d\n", sizeof(uid_t));
	printf("gid_t=%d\n", sizeof(gid_t));
	printf("dev_t=%d\n", sizeof(dev_t));
	printf("off_t=%d\n", sizeof(off_t));
	printf("blksize_t=%d\n", sizeof(blksize_t));
	printf("blkcnt_t=%d\n", sizeof(blkcnt_t));
	printf("clockid_t=%d\n", sizeof(clockid_t));

}
