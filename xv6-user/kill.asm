
xv6-user/_kill:     file format elf64-littleriscv


Disassembly of section .text:

0000000000000000 <main>:
#include "include/stat.h"
#include "xv6-user/user.h"

int
main(int argc, char **argv)
{
   0:	1101                	addi	sp,sp,-32
   2:	ec06                	sd	ra,24(sp)
   4:	e822                	sd	s0,16(sp)
   6:	1000                	addi	s0,sp,32
  int i;

  if(argc < 2){
   8:	4785                	li	a5,1
   a:	02a7d963          	bge	a5,a0,3c <main+0x3c>
   e:	e426                	sd	s1,8(sp)
  10:	e04a                	sd	s2,0(sp)
  12:	00858493          	addi	s1,a1,8
  16:	ffe5091b          	addiw	s2,a0,-2
  1a:	02091793          	slli	a5,s2,0x20
  1e:	01d7d913          	srli	s2,a5,0x1d
  22:	05c1                	addi	a1,a1,16
  24:	992e                	add	s2,s2,a1
    fprintf(2, "usage: kill pid...\n");
    exit(1);
  }
  for(i=1; i<argc; i++)
    kill(atoi(argv[i]));
  26:	6088                	ld	a0,0(s1)
  28:	1c8000ef          	jal	1f0 <atoi>
  2c:	2f6000ef          	jal	322 <kill>
  for(i=1; i<argc; i++)
  30:	04a1                	addi	s1,s1,8
  32:	ff249ae3          	bne	s1,s2,26 <main+0x26>
  exit(0);
  36:	4501                	li	a0,0
  38:	2ba000ef          	jal	2f2 <exit>
  3c:	e426                	sd	s1,8(sp)
  3e:	e04a                	sd	s2,0(sp)
    fprintf(2, "usage: kill pid...\n");
  40:	00001597          	auipc	a1,0x1
  44:	86058593          	addi	a1,a1,-1952 # 8a0 <malloc+0xf4>
  48:	4509                	li	a0,2
  4a:	680000ef          	jal	6ca <fprintf>
    exit(1);
  4e:	4505                	li	a0,1
  50:	2a2000ef          	jal	2f2 <exit>

0000000000000054 <start>:
//
// wrapper so that it's OK if main() does not call exit().
//
void
start()
{
  54:	1141                	addi	sp,sp,-16
  56:	e406                	sd	ra,8(sp)
  58:	e022                	sd	s0,0(sp)
  5a:	0800                	addi	s0,sp,16
  extern int main();
  main();
  5c:	fa5ff0ef          	jal	0 <main>
  exit(0);
  60:	4501                	li	a0,0
  62:	290000ef          	jal	2f2 <exit>

0000000000000066 <strcpy>:
}

char*
strcpy(char *s, const char *t)
{
  66:	1141                	addi	sp,sp,-16
  68:	e406                	sd	ra,8(sp)
  6a:	e022                	sd	s0,0(sp)
  6c:	0800                	addi	s0,sp,16
  char *os;

  os = s;
  while((*s++ = *t++) != 0)
  6e:	87aa                	mv	a5,a0
  70:	0585                	addi	a1,a1,1
  72:	0785                	addi	a5,a5,1
  74:	fff5c703          	lbu	a4,-1(a1)
  78:	fee78fa3          	sb	a4,-1(a5)
  7c:	fb75                	bnez	a4,70 <strcpy+0xa>
    ;
  return os;
}
  7e:	60a2                	ld	ra,8(sp)
  80:	6402                	ld	s0,0(sp)
  82:	0141                	addi	sp,sp,16
  84:	8082                	ret

0000000000000086 <strcmp>:

int
strcmp(const char *p, const char *q)
{
  86:	1141                	addi	sp,sp,-16
  88:	e406                	sd	ra,8(sp)
  8a:	e022                	sd	s0,0(sp)
  8c:	0800                	addi	s0,sp,16
  while(*p && *p == *q)
  8e:	00054783          	lbu	a5,0(a0)
  92:	cb91                	beqz	a5,a6 <strcmp+0x20>
  94:	0005c703          	lbu	a4,0(a1)
  98:	00f71763          	bne	a4,a5,a6 <strcmp+0x20>
    p++, q++;
  9c:	0505                	addi	a0,a0,1
  9e:	0585                	addi	a1,a1,1
  while(*p && *p == *q)
  a0:	00054783          	lbu	a5,0(a0)
  a4:	fbe5                	bnez	a5,94 <strcmp+0xe>
  return (uchar)*p - (uchar)*q;
  a6:	0005c503          	lbu	a0,0(a1)
}
  aa:	40a7853b          	subw	a0,a5,a0
  ae:	60a2                	ld	ra,8(sp)
  b0:	6402                	ld	s0,0(sp)
  b2:	0141                	addi	sp,sp,16
  b4:	8082                	ret

00000000000000b6 <strlen>:

uint
strlen(const char *s)
{
  b6:	1141                	addi	sp,sp,-16
  b8:	e406                	sd	ra,8(sp)
  ba:	e022                	sd	s0,0(sp)
  bc:	0800                	addi	s0,sp,16
  int n;

  for(n = 0; s[n]; n++)
  be:	00054783          	lbu	a5,0(a0)
  c2:	cf99                	beqz	a5,e0 <strlen+0x2a>
  c4:	0505                	addi	a0,a0,1
  c6:	87aa                	mv	a5,a0
  c8:	86be                	mv	a3,a5
  ca:	0785                	addi	a5,a5,1
  cc:	fff7c703          	lbu	a4,-1(a5)
  d0:	ff65                	bnez	a4,c8 <strlen+0x12>
  d2:	40a6853b          	subw	a0,a3,a0
  d6:	2505                	addiw	a0,a0,1
    ;
  return n;
}
  d8:	60a2                	ld	ra,8(sp)
  da:	6402                	ld	s0,0(sp)
  dc:	0141                	addi	sp,sp,16
  de:	8082                	ret
  for(n = 0; s[n]; n++)
  e0:	4501                	li	a0,0
  e2:	bfdd                	j	d8 <strlen+0x22>

00000000000000e4 <memset>:

void*
memset(void *dst, int c, uint n)
{
  e4:	1141                	addi	sp,sp,-16
  e6:	e406                	sd	ra,8(sp)
  e8:	e022                	sd	s0,0(sp)
  ea:	0800                	addi	s0,sp,16
  char *cdst = (char *) dst;
  int i;
  for(i = 0; i < n; i++){
  ec:	ca19                	beqz	a2,102 <memset+0x1e>
  ee:	87aa                	mv	a5,a0
  f0:	1602                	slli	a2,a2,0x20
  f2:	9201                	srli	a2,a2,0x20
  f4:	00a60733          	add	a4,a2,a0
    cdst[i] = c;
  f8:	00b78023          	sb	a1,0(a5)
  for(i = 0; i < n; i++){
  fc:	0785                	addi	a5,a5,1
  fe:	fee79de3          	bne	a5,a4,f8 <memset+0x14>
  }
  return dst;
}
 102:	60a2                	ld	ra,8(sp)
 104:	6402                	ld	s0,0(sp)
 106:	0141                	addi	sp,sp,16
 108:	8082                	ret

000000000000010a <strchr>:

char*
strchr(const char *s, char c)
{
 10a:	1141                	addi	sp,sp,-16
 10c:	e406                	sd	ra,8(sp)
 10e:	e022                	sd	s0,0(sp)
 110:	0800                	addi	s0,sp,16
  for(; *s; s++)
 112:	00054783          	lbu	a5,0(a0)
 116:	cf81                	beqz	a5,12e <strchr+0x24>
    if(*s == c)
 118:	00f58763          	beq	a1,a5,126 <strchr+0x1c>
  for(; *s; s++)
 11c:	0505                	addi	a0,a0,1
 11e:	00054783          	lbu	a5,0(a0)
 122:	fbfd                	bnez	a5,118 <strchr+0xe>
      return (char*)s;
  return 0;
 124:	4501                	li	a0,0
}
 126:	60a2                	ld	ra,8(sp)
 128:	6402                	ld	s0,0(sp)
 12a:	0141                	addi	sp,sp,16
 12c:	8082                	ret
  return 0;
 12e:	4501                	li	a0,0
 130:	bfdd                	j	126 <strchr+0x1c>

0000000000000132 <gets>:

char*
gets(char *buf, int max)
{
 132:	7159                	addi	sp,sp,-112
 134:	f486                	sd	ra,104(sp)
 136:	f0a2                	sd	s0,96(sp)
 138:	eca6                	sd	s1,88(sp)
 13a:	e8ca                	sd	s2,80(sp)
 13c:	e4ce                	sd	s3,72(sp)
 13e:	e0d2                	sd	s4,64(sp)
 140:	fc56                	sd	s5,56(sp)
 142:	f85a                	sd	s6,48(sp)
 144:	f45e                	sd	s7,40(sp)
 146:	f062                	sd	s8,32(sp)
 148:	ec66                	sd	s9,24(sp)
 14a:	e86a                	sd	s10,16(sp)
 14c:	1880                	addi	s0,sp,112
 14e:	8caa                	mv	s9,a0
 150:	8a2e                	mv	s4,a1
  int i, cc;
  char c;

  for(i=0; i+1 < max; ){
 152:	892a                	mv	s2,a0
 154:	4481                	li	s1,0
    cc = read(0, &c, 1);
 156:	f9f40b13          	addi	s6,s0,-97
 15a:	4a85                	li	s5,1
    if(cc < 1)
      break;
    buf[i++] = c;
    if(c == '\n' || c == '\r')
 15c:	4ba9                	li	s7,10
 15e:	4c35                	li	s8,13
  for(i=0; i+1 < max; ){
 160:	8d26                	mv	s10,s1
 162:	0014899b          	addiw	s3,s1,1
 166:	84ce                	mv	s1,s3
 168:	0349d563          	bge	s3,s4,192 <gets+0x60>
    cc = read(0, &c, 1);
 16c:	8656                	mv	a2,s5
 16e:	85da                	mv	a1,s6
 170:	4501                	li	a0,0
 172:	198000ef          	jal	30a <read>
    if(cc < 1)
 176:	00a05e63          	blez	a0,192 <gets+0x60>
    buf[i++] = c;
 17a:	f9f44783          	lbu	a5,-97(s0)
 17e:	00f90023          	sb	a5,0(s2)
    if(c == '\n' || c == '\r')
 182:	01778763          	beq	a5,s7,190 <gets+0x5e>
 186:	0905                	addi	s2,s2,1
 188:	fd879ce3          	bne	a5,s8,160 <gets+0x2e>
    buf[i++] = c;
 18c:	8d4e                	mv	s10,s3
 18e:	a011                	j	192 <gets+0x60>
 190:	8d4e                	mv	s10,s3
      break;
  }
  buf[i] = '\0';
 192:	9d66                	add	s10,s10,s9
 194:	000d0023          	sb	zero,0(s10)
  return buf;
}
 198:	8566                	mv	a0,s9
 19a:	70a6                	ld	ra,104(sp)
 19c:	7406                	ld	s0,96(sp)
 19e:	64e6                	ld	s1,88(sp)
 1a0:	6946                	ld	s2,80(sp)
 1a2:	69a6                	ld	s3,72(sp)
 1a4:	6a06                	ld	s4,64(sp)
 1a6:	7ae2                	ld	s5,56(sp)
 1a8:	7b42                	ld	s6,48(sp)
 1aa:	7ba2                	ld	s7,40(sp)
 1ac:	7c02                	ld	s8,32(sp)
 1ae:	6ce2                	ld	s9,24(sp)
 1b0:	6d42                	ld	s10,16(sp)
 1b2:	6165                	addi	sp,sp,112
 1b4:	8082                	ret

00000000000001b6 <stat>:

int
stat(const char *n, struct stat *st)
{
 1b6:	1101                	addi	sp,sp,-32
 1b8:	ec06                	sd	ra,24(sp)
 1ba:	e822                	sd	s0,16(sp)
 1bc:	e04a                	sd	s2,0(sp)
 1be:	1000                	addi	s0,sp,32
 1c0:	892e                	mv	s2,a1
  int fd;
  int r;

  fd = open(n, O_RDONLY);
 1c2:	4581                	li	a1,0
 1c4:	16e000ef          	jal	332 <open>
  if(fd < 0)
 1c8:	02054263          	bltz	a0,1ec <stat+0x36>
 1cc:	e426                	sd	s1,8(sp)
 1ce:	84aa                	mv	s1,a0
    return -1;
  r = fstat(fd, st);
 1d0:	85ca                	mv	a1,s2
 1d2:	178000ef          	jal	34a <fstat>
 1d6:	892a                	mv	s2,a0
  close(fd);
 1d8:	8526                	mv	a0,s1
 1da:	140000ef          	jal	31a <close>
  return r;
 1de:	64a2                	ld	s1,8(sp)
}
 1e0:	854a                	mv	a0,s2
 1e2:	60e2                	ld	ra,24(sp)
 1e4:	6442                	ld	s0,16(sp)
 1e6:	6902                	ld	s2,0(sp)
 1e8:	6105                	addi	sp,sp,32
 1ea:	8082                	ret
    return -1;
 1ec:	597d                	li	s2,-1
 1ee:	bfcd                	j	1e0 <stat+0x2a>

00000000000001f0 <atoi>:

int
atoi(const char *s)
{
 1f0:	1141                	addi	sp,sp,-16
 1f2:	e406                	sd	ra,8(sp)
 1f4:	e022                	sd	s0,0(sp)
 1f6:	0800                	addi	s0,sp,16
  int n;

  n = 0;
  while('0' <= *s && *s <= '9')
 1f8:	00054683          	lbu	a3,0(a0)
 1fc:	fd06879b          	addiw	a5,a3,-48
 200:	0ff7f793          	zext.b	a5,a5
 204:	4625                	li	a2,9
 206:	02f66963          	bltu	a2,a5,238 <atoi+0x48>
 20a:	872a                	mv	a4,a0
  n = 0;
 20c:	4501                	li	a0,0
    n = n*10 + *s++ - '0';
 20e:	0705                	addi	a4,a4,1
 210:	0025179b          	slliw	a5,a0,0x2
 214:	9fa9                	addw	a5,a5,a0
 216:	0017979b          	slliw	a5,a5,0x1
 21a:	9fb5                	addw	a5,a5,a3
 21c:	fd07851b          	addiw	a0,a5,-48
  while('0' <= *s && *s <= '9')
 220:	00074683          	lbu	a3,0(a4)
 224:	fd06879b          	addiw	a5,a3,-48
 228:	0ff7f793          	zext.b	a5,a5
 22c:	fef671e3          	bgeu	a2,a5,20e <atoi+0x1e>
  return n;
}
 230:	60a2                	ld	ra,8(sp)
 232:	6402                	ld	s0,0(sp)
 234:	0141                	addi	sp,sp,16
 236:	8082                	ret
  n = 0;
 238:	4501                	li	a0,0
 23a:	bfdd                	j	230 <atoi+0x40>

000000000000023c <memmove>:

void*
memmove(void *vdst, const void *vsrc, int n)
{
 23c:	1141                	addi	sp,sp,-16
 23e:	e406                	sd	ra,8(sp)
 240:	e022                	sd	s0,0(sp)
 242:	0800                	addi	s0,sp,16
  char *dst;
  const char *src;

  dst = vdst;
  src = vsrc;
  if (src > dst) {
 244:	02b57563          	bgeu	a0,a1,26e <memmove+0x32>
    while(n-- > 0)
 248:	00c05f63          	blez	a2,266 <memmove+0x2a>
 24c:	1602                	slli	a2,a2,0x20
 24e:	9201                	srli	a2,a2,0x20
 250:	00c507b3          	add	a5,a0,a2
  dst = vdst;
 254:	872a                	mv	a4,a0
      *dst++ = *src++;
 256:	0585                	addi	a1,a1,1
 258:	0705                	addi	a4,a4,1
 25a:	fff5c683          	lbu	a3,-1(a1)
 25e:	fed70fa3          	sb	a3,-1(a4)
    while(n-- > 0)
 262:	fee79ae3          	bne	a5,a4,256 <memmove+0x1a>
    src += n;
    while(n-- > 0)
      *--dst = *--src;
  }
  return vdst;
}
 266:	60a2                	ld	ra,8(sp)
 268:	6402                	ld	s0,0(sp)
 26a:	0141                	addi	sp,sp,16
 26c:	8082                	ret
    dst += n;
 26e:	00c50733          	add	a4,a0,a2
    src += n;
 272:	95b2                	add	a1,a1,a2
    while(n-- > 0)
 274:	fec059e3          	blez	a2,266 <memmove+0x2a>
 278:	fff6079b          	addiw	a5,a2,-1
 27c:	1782                	slli	a5,a5,0x20
 27e:	9381                	srli	a5,a5,0x20
 280:	fff7c793          	not	a5,a5
 284:	97ba                	add	a5,a5,a4
      *--dst = *--src;
 286:	15fd                	addi	a1,a1,-1
 288:	177d                	addi	a4,a4,-1
 28a:	0005c683          	lbu	a3,0(a1)
 28e:	00d70023          	sb	a3,0(a4)
    while(n-- > 0)
 292:	fef71ae3          	bne	a4,a5,286 <memmove+0x4a>
 296:	bfc1                	j	266 <memmove+0x2a>

0000000000000298 <memcmp>:

int
memcmp(const void *s1, const void *s2, uint n)
{
 298:	1141                	addi	sp,sp,-16
 29a:	e406                	sd	ra,8(sp)
 29c:	e022                	sd	s0,0(sp)
 29e:	0800                	addi	s0,sp,16
  const char *p1 = s1, *p2 = s2;
  while (n-- > 0) {
 2a0:	ca0d                	beqz	a2,2d2 <memcmp+0x3a>
 2a2:	fff6069b          	addiw	a3,a2,-1
 2a6:	1682                	slli	a3,a3,0x20
 2a8:	9281                	srli	a3,a3,0x20
 2aa:	0685                	addi	a3,a3,1
 2ac:	96aa                	add	a3,a3,a0
    if (*p1 != *p2) {
 2ae:	00054783          	lbu	a5,0(a0)
 2b2:	0005c703          	lbu	a4,0(a1)
 2b6:	00e79863          	bne	a5,a4,2c6 <memcmp+0x2e>
      return *p1 - *p2;
    }
    p1++;
 2ba:	0505                	addi	a0,a0,1
    p2++;
 2bc:	0585                	addi	a1,a1,1
  while (n-- > 0) {
 2be:	fed518e3          	bne	a0,a3,2ae <memcmp+0x16>
  }
  return 0;
 2c2:	4501                	li	a0,0
 2c4:	a019                	j	2ca <memcmp+0x32>
      return *p1 - *p2;
 2c6:	40e7853b          	subw	a0,a5,a4
}
 2ca:	60a2                	ld	ra,8(sp)
 2cc:	6402                	ld	s0,0(sp)
 2ce:	0141                	addi	sp,sp,16
 2d0:	8082                	ret
  return 0;
 2d2:	4501                	li	a0,0
 2d4:	bfdd                	j	2ca <memcmp+0x32>

00000000000002d6 <memcpy>:

void *
memcpy(void *dst, const void *src, uint n)
{
 2d6:	1141                	addi	sp,sp,-16
 2d8:	e406                	sd	ra,8(sp)
 2da:	e022                	sd	s0,0(sp)
 2dc:	0800                	addi	s0,sp,16
  return memmove(dst, src, n);
 2de:	f5fff0ef          	jal	23c <memmove>
}
 2e2:	60a2                	ld	ra,8(sp)
 2e4:	6402                	ld	s0,0(sp)
 2e6:	0141                	addi	sp,sp,16
 2e8:	8082                	ret

00000000000002ea <fork>:
# generated by usys.pl - do not edit
#include "include/syscall.h"
.global fork
fork:
 li a7, SYS_fork
 2ea:	4885                	li	a7,1
 ecall
 2ec:	00000073          	ecall
 ret
 2f0:	8082                	ret

00000000000002f2 <exit>:
.global exit
exit:
 li a7, SYS_exit
 2f2:	4889                	li	a7,2
 ecall
 2f4:	00000073          	ecall
 ret
 2f8:	8082                	ret

00000000000002fa <wait>:
.global wait
wait:
 li a7, SYS_wait
 2fa:	488d                	li	a7,3
 ecall
 2fc:	00000073          	ecall
 ret
 300:	8082                	ret

0000000000000302 <pipe>:
.global pipe
pipe:
 li a7, SYS_pipe
 302:	4891                	li	a7,4
 ecall
 304:	00000073          	ecall
 ret
 308:	8082                	ret

000000000000030a <read>:
.global read
read:
 li a7, SYS_read
 30a:	4895                	li	a7,5
 ecall
 30c:	00000073          	ecall
 ret
 310:	8082                	ret

0000000000000312 <write>:
.global write
write:
 li a7, SYS_write
 312:	48c1                	li	a7,16
 ecall
 314:	00000073          	ecall
 ret
 318:	8082                	ret

000000000000031a <close>:
.global close
close:
 li a7, SYS_close
 31a:	48d5                	li	a7,21
 ecall
 31c:	00000073          	ecall
 ret
 320:	8082                	ret

0000000000000322 <kill>:
.global kill
kill:
 li a7, SYS_kill
 322:	4899                	li	a7,6
 ecall
 324:	00000073          	ecall
 ret
 328:	8082                	ret

000000000000032a <exec>:
.global exec
exec:
 li a7, SYS_exec
 32a:	489d                	li	a7,7
 ecall
 32c:	00000073          	ecall
 ret
 330:	8082                	ret

0000000000000332 <open>:
.global open
open:
 li a7, SYS_open
 332:	48bd                	li	a7,15
 ecall
 334:	00000073          	ecall
 ret
 338:	8082                	ret

000000000000033a <mknod>:
.global mknod
mknod:
 li a7, SYS_mknod
 33a:	48c5                	li	a7,17
 ecall
 33c:	00000073          	ecall
 ret
 340:	8082                	ret

0000000000000342 <unlink>:
.global unlink
unlink:
 li a7, SYS_unlink
 342:	48c9                	li	a7,18
 ecall
 344:	00000073          	ecall
 ret
 348:	8082                	ret

000000000000034a <fstat>:
.global fstat
fstat:
 li a7, SYS_fstat
 34a:	48a1                	li	a7,8
 ecall
 34c:	00000073          	ecall
 ret
 350:	8082                	ret

0000000000000352 <link>:
.global link
link:
 li a7, SYS_link
 352:	48cd                	li	a7,19
 ecall
 354:	00000073          	ecall
 ret
 358:	8082                	ret

000000000000035a <mkdir>:
.global mkdir
mkdir:
 li a7, SYS_mkdir
 35a:	48d1                	li	a7,20
 ecall
 35c:	00000073          	ecall
 ret
 360:	8082                	ret

0000000000000362 <chdir>:
.global chdir
chdir:
 li a7, SYS_chdir
 362:	48a5                	li	a7,9
 ecall
 364:	00000073          	ecall
 ret
 368:	8082                	ret

000000000000036a <dup>:
.global dup
dup:
 li a7, SYS_dup
 36a:	48a9                	li	a7,10
 ecall
 36c:	00000073          	ecall
 ret
 370:	8082                	ret

0000000000000372 <getpid>:
.global getpid
getpid:
 li a7, SYS_getpid
 372:	48ad                	li	a7,11
 ecall
 374:	00000073          	ecall
 ret
 378:	8082                	ret

000000000000037a <sbrk>:
.global sbrk
sbrk:
 li a7, SYS_sbrk
 37a:	48b1                	li	a7,12
 ecall
 37c:	00000073          	ecall
 ret
 380:	8082                	ret

0000000000000382 <sleep>:
.global sleep
sleep:
 li a7, SYS_sleep
 382:	48b5                	li	a7,13
 ecall
 384:	00000073          	ecall
 ret
 388:	8082                	ret

000000000000038a <uptime>:
.global uptime
uptime:
 li a7, SYS_uptime
 38a:	48b9                	li	a7,14
 ecall
 38c:	00000073          	ecall
 ret
 390:	8082                	ret

0000000000000392 <putc>:

static char digits[] = "0123456789ABCDEF";

static void
putc(int fd, char c)
{
 392:	1101                	addi	sp,sp,-32
 394:	ec06                	sd	ra,24(sp)
 396:	e822                	sd	s0,16(sp)
 398:	1000                	addi	s0,sp,32
 39a:	feb407a3          	sb	a1,-17(s0)
  write(fd, &c, 1);
 39e:	4605                	li	a2,1
 3a0:	fef40593          	addi	a1,s0,-17
 3a4:	f6fff0ef          	jal	312 <write>
}
 3a8:	60e2                	ld	ra,24(sp)
 3aa:	6442                	ld	s0,16(sp)
 3ac:	6105                	addi	sp,sp,32
 3ae:	8082                	ret

00000000000003b0 <printint>:

static void
printint(int fd, int xx, int base, int sgn)
{
 3b0:	7139                	addi	sp,sp,-64
 3b2:	fc06                	sd	ra,56(sp)
 3b4:	f822                	sd	s0,48(sp)
 3b6:	f426                	sd	s1,40(sp)
 3b8:	f04a                	sd	s2,32(sp)
 3ba:	ec4e                	sd	s3,24(sp)
 3bc:	0080                	addi	s0,sp,64
 3be:	892a                	mv	s2,a0
  char buf[16];
  int i, neg;
  uint x;

  neg = 0;
  if(sgn && xx < 0){
 3c0:	c299                	beqz	a3,3c6 <printint+0x16>
 3c2:	0605ce63          	bltz	a1,43e <printint+0x8e>
  neg = 0;
 3c6:	4e01                	li	t3,0
    x = -xx;
  } else {
    x = xx;
  }

  i = 0;
 3c8:	fc040313          	addi	t1,s0,-64
  neg = 0;
 3cc:	869a                	mv	a3,t1
  i = 0;
 3ce:	4781                	li	a5,0
  do{
    buf[i++] = digits[x % base];
 3d0:	00000817          	auipc	a6,0x0
 3d4:	4f080813          	addi	a6,a6,1264 # 8c0 <digits>
 3d8:	88be                	mv	a7,a5
 3da:	0017851b          	addiw	a0,a5,1
 3de:	87aa                	mv	a5,a0
 3e0:	02c5f73b          	remuw	a4,a1,a2
 3e4:	1702                	slli	a4,a4,0x20
 3e6:	9301                	srli	a4,a4,0x20
 3e8:	9742                	add	a4,a4,a6
 3ea:	00074703          	lbu	a4,0(a4)
 3ee:	00e68023          	sb	a4,0(a3)
  }while((x /= base) != 0);
 3f2:	872e                	mv	a4,a1
 3f4:	02c5d5bb          	divuw	a1,a1,a2
 3f8:	0685                	addi	a3,a3,1
 3fa:	fcc77fe3          	bgeu	a4,a2,3d8 <printint+0x28>
  if(neg)
 3fe:	000e0c63          	beqz	t3,416 <printint+0x66>
    buf[i++] = '-';
 402:	fd050793          	addi	a5,a0,-48
 406:	00878533          	add	a0,a5,s0
 40a:	02d00793          	li	a5,45
 40e:	fef50823          	sb	a5,-16(a0)
 412:	0028879b          	addiw	a5,a7,2

  while(--i >= 0)
 416:	fff7899b          	addiw	s3,a5,-1
 41a:	006784b3          	add	s1,a5,t1
    putc(fd, buf[i]);
 41e:	fff4c583          	lbu	a1,-1(s1)
 422:	854a                	mv	a0,s2
 424:	f6fff0ef          	jal	392 <putc>
  while(--i >= 0)
 428:	39fd                	addiw	s3,s3,-1
 42a:	14fd                	addi	s1,s1,-1
 42c:	fe09d9e3          	bgez	s3,41e <printint+0x6e>
}
 430:	70e2                	ld	ra,56(sp)
 432:	7442                	ld	s0,48(sp)
 434:	74a2                	ld	s1,40(sp)
 436:	7902                	ld	s2,32(sp)
 438:	69e2                	ld	s3,24(sp)
 43a:	6121                	addi	sp,sp,64
 43c:	8082                	ret
    x = -xx;
 43e:	40b005bb          	negw	a1,a1
    neg = 1;
 442:	4e05                	li	t3,1
    x = -xx;
 444:	b751                	j	3c8 <printint+0x18>

0000000000000446 <vprintf>:
}

// Print to the given fd. Only understands %d, %x, %p, %s.
void
vprintf(int fd, const char *fmt, va_list ap)
{
 446:	711d                	addi	sp,sp,-96
 448:	ec86                	sd	ra,88(sp)
 44a:	e8a2                	sd	s0,80(sp)
 44c:	e4a6                	sd	s1,72(sp)
 44e:	1080                	addi	s0,sp,96
  char *s;
  int c0, c1, c2, i, state;

  state = 0;
  for(i = 0; fmt[i]; i++){
 450:	0005c483          	lbu	s1,0(a1)
 454:	26048663          	beqz	s1,6c0 <vprintf+0x27a>
 458:	e0ca                	sd	s2,64(sp)
 45a:	fc4e                	sd	s3,56(sp)
 45c:	f852                	sd	s4,48(sp)
 45e:	f456                	sd	s5,40(sp)
 460:	f05a                	sd	s6,32(sp)
 462:	ec5e                	sd	s7,24(sp)
 464:	e862                	sd	s8,16(sp)
 466:	e466                	sd	s9,8(sp)
 468:	8b2a                	mv	s6,a0
 46a:	8a2e                	mv	s4,a1
 46c:	8bb2                	mv	s7,a2
  state = 0;
 46e:	4981                	li	s3,0
  for(i = 0; fmt[i]; i++){
 470:	4901                	li	s2,0
 472:	4701                	li	a4,0
      if(c0 == '%'){
        state = '%';
      } else {
        putc(fd, c0);
      }
    } else if(state == '%'){
 474:	02500a93          	li	s5,37
      c1 = c2 = 0;
      if(c0) c1 = fmt[i+1] & 0xff;
      if(c1) c2 = fmt[i+2] & 0xff;
      if(c0 == 'd'){
 478:	06400c13          	li	s8,100
        printint(fd, va_arg(ap, int), 10, 1);
      } else if(c0 == 'l' && c1 == 'd'){
 47c:	06c00c93          	li	s9,108
 480:	a00d                	j	4a2 <vprintf+0x5c>
        putc(fd, c0);
 482:	85a6                	mv	a1,s1
 484:	855a                	mv	a0,s6
 486:	f0dff0ef          	jal	392 <putc>
 48a:	a019                	j	490 <vprintf+0x4a>
    } else if(state == '%'){
 48c:	03598363          	beq	s3,s5,4b2 <vprintf+0x6c>
  for(i = 0; fmt[i]; i++){
 490:	0019079b          	addiw	a5,s2,1
 494:	893e                	mv	s2,a5
 496:	873e                	mv	a4,a5
 498:	97d2                	add	a5,a5,s4
 49a:	0007c483          	lbu	s1,0(a5)
 49e:	20048963          	beqz	s1,6b0 <vprintf+0x26a>
    c0 = fmt[i] & 0xff;
 4a2:	0004879b          	sext.w	a5,s1
    if(state == 0){
 4a6:	fe0993e3          	bnez	s3,48c <vprintf+0x46>
      if(c0 == '%'){
 4aa:	fd579ce3          	bne	a5,s5,482 <vprintf+0x3c>
        state = '%';
 4ae:	89be                	mv	s3,a5
 4b0:	b7c5                	j	490 <vprintf+0x4a>
      if(c0) c1 = fmt[i+1] & 0xff;
 4b2:	00ea06b3          	add	a3,s4,a4
 4b6:	0016c683          	lbu	a3,1(a3)
      c1 = c2 = 0;
 4ba:	8636                	mv	a2,a3
      if(c1) c2 = fmt[i+2] & 0xff;
 4bc:	c681                	beqz	a3,4c4 <vprintf+0x7e>
 4be:	9752                	add	a4,a4,s4
 4c0:	00274603          	lbu	a2,2(a4)
      if(c0 == 'd'){
 4c4:	03878e63          	beq	a5,s8,500 <vprintf+0xba>
      } else if(c0 == 'l' && c1 == 'd'){
 4c8:	05978863          	beq	a5,s9,518 <vprintf+0xd2>
        printint(fd, va_arg(ap, uint64), 10, 1);
        i += 1;
      } else if(c0 == 'l' && c1 == 'l' && c2 == 'd'){
        printint(fd, va_arg(ap, uint64), 10, 1);
        i += 2;
      } else if(c0 == 'u'){
 4cc:	07500713          	li	a4,117
 4d0:	0ee78263          	beq	a5,a4,5b4 <vprintf+0x16e>
        printint(fd, va_arg(ap, uint64), 10, 0);
        i += 1;
      } else if(c0 == 'l' && c1 == 'l' && c2 == 'u'){
        printint(fd, va_arg(ap, uint64), 10, 0);
        i += 2;
      } else if(c0 == 'x'){
 4d4:	07800713          	li	a4,120
 4d8:	12e78463          	beq	a5,a4,600 <vprintf+0x1ba>
        printint(fd, va_arg(ap, uint64), 16, 0);
        i += 1;
      } else if(c0 == 'l' && c1 == 'l' && c2 == 'x'){
        printint(fd, va_arg(ap, uint64), 16, 0);
        i += 2;
      } else if(c0 == 'p'){
 4dc:	07000713          	li	a4,112
 4e0:	14e78963          	beq	a5,a4,632 <vprintf+0x1ec>
        printptr(fd, va_arg(ap, uint64));
      } else if(c0 == 's'){
 4e4:	07300713          	li	a4,115
 4e8:	18e78863          	beq	a5,a4,678 <vprintf+0x232>
        if((s = va_arg(ap, char*)) == 0)
          s = "(null)";
        for(; *s; s++)
          putc(fd, *s);
      } else if(c0 == '%'){
 4ec:	02500713          	li	a4,37
 4f0:	04e79463          	bne	a5,a4,538 <vprintf+0xf2>
        putc(fd, '%');
 4f4:	85ba                	mv	a1,a4
 4f6:	855a                	mv	a0,s6
 4f8:	e9bff0ef          	jal	392 <putc>
        // Unknown % sequence.  Print it to draw attention.
        putc(fd, '%');
        putc(fd, c);
      }
#endif
      state = 0;
 4fc:	4981                	li	s3,0
 4fe:	bf49                	j	490 <vprintf+0x4a>
        printint(fd, va_arg(ap, int), 10, 1);
 500:	008b8493          	addi	s1,s7,8
 504:	4685                	li	a3,1
 506:	4629                	li	a2,10
 508:	000ba583          	lw	a1,0(s7)
 50c:	855a                	mv	a0,s6
 50e:	ea3ff0ef          	jal	3b0 <printint>
 512:	8ba6                	mv	s7,s1
      state = 0;
 514:	4981                	li	s3,0
 516:	bfad                	j	490 <vprintf+0x4a>
      } else if(c0 == 'l' && c1 == 'd'){
 518:	06400793          	li	a5,100
 51c:	02f68963          	beq	a3,a5,54e <vprintf+0x108>
      } else if(c0 == 'l' && c1 == 'l' && c2 == 'd'){
 520:	06c00793          	li	a5,108
 524:	04f68263          	beq	a3,a5,568 <vprintf+0x122>
      } else if(c0 == 'l' && c1 == 'u'){
 528:	07500793          	li	a5,117
 52c:	0af68063          	beq	a3,a5,5cc <vprintf+0x186>
      } else if(c0 == 'l' && c1 == 'x'){
 530:	07800793          	li	a5,120
 534:	0ef68263          	beq	a3,a5,618 <vprintf+0x1d2>
        putc(fd, '%');
 538:	02500593          	li	a1,37
 53c:	855a                	mv	a0,s6
 53e:	e55ff0ef          	jal	392 <putc>
        putc(fd, c0);
 542:	85a6                	mv	a1,s1
 544:	855a                	mv	a0,s6
 546:	e4dff0ef          	jal	392 <putc>
      state = 0;
 54a:	4981                	li	s3,0
 54c:	b791                	j	490 <vprintf+0x4a>
        printint(fd, va_arg(ap, uint64), 10, 1);
 54e:	008b8493          	addi	s1,s7,8
 552:	4685                	li	a3,1
 554:	4629                	li	a2,10
 556:	000ba583          	lw	a1,0(s7)
 55a:	855a                	mv	a0,s6
 55c:	e55ff0ef          	jal	3b0 <printint>
        i += 1;
 560:	2905                	addiw	s2,s2,1
        printint(fd, va_arg(ap, uint64), 10, 1);
 562:	8ba6                	mv	s7,s1
      state = 0;
 564:	4981                	li	s3,0
        i += 1;
 566:	b72d                	j	490 <vprintf+0x4a>
      } else if(c0 == 'l' && c1 == 'l' && c2 == 'd'){
 568:	06400793          	li	a5,100
 56c:	02f60763          	beq	a2,a5,59a <vprintf+0x154>
      } else if(c0 == 'l' && c1 == 'l' && c2 == 'u'){
 570:	07500793          	li	a5,117
 574:	06f60963          	beq	a2,a5,5e6 <vprintf+0x1a0>
      } else if(c0 == 'l' && c1 == 'l' && c2 == 'x'){
 578:	07800793          	li	a5,120
 57c:	faf61ee3          	bne	a2,a5,538 <vprintf+0xf2>
        printint(fd, va_arg(ap, uint64), 16, 0);
 580:	008b8493          	addi	s1,s7,8
 584:	4681                	li	a3,0
 586:	4641                	li	a2,16
 588:	000ba583          	lw	a1,0(s7)
 58c:	855a                	mv	a0,s6
 58e:	e23ff0ef          	jal	3b0 <printint>
        i += 2;
 592:	2909                	addiw	s2,s2,2
        printint(fd, va_arg(ap, uint64), 16, 0);
 594:	8ba6                	mv	s7,s1
      state = 0;
 596:	4981                	li	s3,0
        i += 2;
 598:	bde5                	j	490 <vprintf+0x4a>
        printint(fd, va_arg(ap, uint64), 10, 1);
 59a:	008b8493          	addi	s1,s7,8
 59e:	4685                	li	a3,1
 5a0:	4629                	li	a2,10
 5a2:	000ba583          	lw	a1,0(s7)
 5a6:	855a                	mv	a0,s6
 5a8:	e09ff0ef          	jal	3b0 <printint>
        i += 2;
 5ac:	2909                	addiw	s2,s2,2
        printint(fd, va_arg(ap, uint64), 10, 1);
 5ae:	8ba6                	mv	s7,s1
      state = 0;
 5b0:	4981                	li	s3,0
        i += 2;
 5b2:	bdf9                	j	490 <vprintf+0x4a>
        printint(fd, va_arg(ap, int), 10, 0);
 5b4:	008b8493          	addi	s1,s7,8
 5b8:	4681                	li	a3,0
 5ba:	4629                	li	a2,10
 5bc:	000ba583          	lw	a1,0(s7)
 5c0:	855a                	mv	a0,s6
 5c2:	defff0ef          	jal	3b0 <printint>
 5c6:	8ba6                	mv	s7,s1
      state = 0;
 5c8:	4981                	li	s3,0
 5ca:	b5d9                	j	490 <vprintf+0x4a>
        printint(fd, va_arg(ap, uint64), 10, 0);
 5cc:	008b8493          	addi	s1,s7,8
 5d0:	4681                	li	a3,0
 5d2:	4629                	li	a2,10
 5d4:	000ba583          	lw	a1,0(s7)
 5d8:	855a                	mv	a0,s6
 5da:	dd7ff0ef          	jal	3b0 <printint>
        i += 1;
 5de:	2905                	addiw	s2,s2,1
        printint(fd, va_arg(ap, uint64), 10, 0);
 5e0:	8ba6                	mv	s7,s1
      state = 0;
 5e2:	4981                	li	s3,0
        i += 1;
 5e4:	b575                	j	490 <vprintf+0x4a>
        printint(fd, va_arg(ap, uint64), 10, 0);
 5e6:	008b8493          	addi	s1,s7,8
 5ea:	4681                	li	a3,0
 5ec:	4629                	li	a2,10
 5ee:	000ba583          	lw	a1,0(s7)
 5f2:	855a                	mv	a0,s6
 5f4:	dbdff0ef          	jal	3b0 <printint>
        i += 2;
 5f8:	2909                	addiw	s2,s2,2
        printint(fd, va_arg(ap, uint64), 10, 0);
 5fa:	8ba6                	mv	s7,s1
      state = 0;
 5fc:	4981                	li	s3,0
        i += 2;
 5fe:	bd49                	j	490 <vprintf+0x4a>
        printint(fd, va_arg(ap, int), 16, 0);
 600:	008b8493          	addi	s1,s7,8
 604:	4681                	li	a3,0
 606:	4641                	li	a2,16
 608:	000ba583          	lw	a1,0(s7)
 60c:	855a                	mv	a0,s6
 60e:	da3ff0ef          	jal	3b0 <printint>
 612:	8ba6                	mv	s7,s1
      state = 0;
 614:	4981                	li	s3,0
 616:	bdad                	j	490 <vprintf+0x4a>
        printint(fd, va_arg(ap, uint64), 16, 0);
 618:	008b8493          	addi	s1,s7,8
 61c:	4681                	li	a3,0
 61e:	4641                	li	a2,16
 620:	000ba583          	lw	a1,0(s7)
 624:	855a                	mv	a0,s6
 626:	d8bff0ef          	jal	3b0 <printint>
        i += 1;
 62a:	2905                	addiw	s2,s2,1
        printint(fd, va_arg(ap, uint64), 16, 0);
 62c:	8ba6                	mv	s7,s1
      state = 0;
 62e:	4981                	li	s3,0
        i += 1;
 630:	b585                	j	490 <vprintf+0x4a>
 632:	e06a                	sd	s10,0(sp)
        printptr(fd, va_arg(ap, uint64));
 634:	008b8d13          	addi	s10,s7,8
 638:	000bb983          	ld	s3,0(s7)
  putc(fd, '0');
 63c:	03000593          	li	a1,48
 640:	855a                	mv	a0,s6
 642:	d51ff0ef          	jal	392 <putc>
  putc(fd, 'x');
 646:	07800593          	li	a1,120
 64a:	855a                	mv	a0,s6
 64c:	d47ff0ef          	jal	392 <putc>
 650:	44c1                	li	s1,16
    putc(fd, digits[x >> (sizeof(uint64) * 8 - 4)]);
 652:	00000b97          	auipc	s7,0x0
 656:	26eb8b93          	addi	s7,s7,622 # 8c0 <digits>
 65a:	03c9d793          	srli	a5,s3,0x3c
 65e:	97de                	add	a5,a5,s7
 660:	0007c583          	lbu	a1,0(a5)
 664:	855a                	mv	a0,s6
 666:	d2dff0ef          	jal	392 <putc>
  for (i = 0; i < (sizeof(uint64) * 2); i++, x <<= 4)
 66a:	0992                	slli	s3,s3,0x4
 66c:	34fd                	addiw	s1,s1,-1
 66e:	f4f5                	bnez	s1,65a <vprintf+0x214>
        printptr(fd, va_arg(ap, uint64));
 670:	8bea                	mv	s7,s10
      state = 0;
 672:	4981                	li	s3,0
 674:	6d02                	ld	s10,0(sp)
 676:	bd29                	j	490 <vprintf+0x4a>
        if((s = va_arg(ap, char*)) == 0)
 678:	008b8993          	addi	s3,s7,8
 67c:	000bb483          	ld	s1,0(s7)
 680:	cc91                	beqz	s1,69c <vprintf+0x256>
        for(; *s; s++)
 682:	0004c583          	lbu	a1,0(s1)
 686:	c195                	beqz	a1,6aa <vprintf+0x264>
          putc(fd, *s);
 688:	855a                	mv	a0,s6
 68a:	d09ff0ef          	jal	392 <putc>
        for(; *s; s++)
 68e:	0485                	addi	s1,s1,1
 690:	0004c583          	lbu	a1,0(s1)
 694:	f9f5                	bnez	a1,688 <vprintf+0x242>
        if((s = va_arg(ap, char*)) == 0)
 696:	8bce                	mv	s7,s3
      state = 0;
 698:	4981                	li	s3,0
 69a:	bbdd                	j	490 <vprintf+0x4a>
          s = "(null)";
 69c:	00000497          	auipc	s1,0x0
 6a0:	21c48493          	addi	s1,s1,540 # 8b8 <malloc+0x10c>
        for(; *s; s++)
 6a4:	02800593          	li	a1,40
 6a8:	b7c5                	j	688 <vprintf+0x242>
        if((s = va_arg(ap, char*)) == 0)
 6aa:	8bce                	mv	s7,s3
      state = 0;
 6ac:	4981                	li	s3,0
 6ae:	b3cd                	j	490 <vprintf+0x4a>
 6b0:	6906                	ld	s2,64(sp)
 6b2:	79e2                	ld	s3,56(sp)
 6b4:	7a42                	ld	s4,48(sp)
 6b6:	7aa2                	ld	s5,40(sp)
 6b8:	7b02                	ld	s6,32(sp)
 6ba:	6be2                	ld	s7,24(sp)
 6bc:	6c42                	ld	s8,16(sp)
 6be:	6ca2                	ld	s9,8(sp)
    }
  }
}
 6c0:	60e6                	ld	ra,88(sp)
 6c2:	6446                	ld	s0,80(sp)
 6c4:	64a6                	ld	s1,72(sp)
 6c6:	6125                	addi	sp,sp,96
 6c8:	8082                	ret

00000000000006ca <fprintf>:

void
fprintf(int fd, const char *fmt, ...)
{
 6ca:	715d                	addi	sp,sp,-80
 6cc:	ec06                	sd	ra,24(sp)
 6ce:	e822                	sd	s0,16(sp)
 6d0:	1000                	addi	s0,sp,32
 6d2:	e010                	sd	a2,0(s0)
 6d4:	e414                	sd	a3,8(s0)
 6d6:	e818                	sd	a4,16(s0)
 6d8:	ec1c                	sd	a5,24(s0)
 6da:	03043023          	sd	a6,32(s0)
 6de:	03143423          	sd	a7,40(s0)
  va_list ap;

  va_start(ap, fmt);
 6e2:	8622                	mv	a2,s0
 6e4:	fe843423          	sd	s0,-24(s0)
  vprintf(fd, fmt, ap);
 6e8:	d5fff0ef          	jal	446 <vprintf>
}
 6ec:	60e2                	ld	ra,24(sp)
 6ee:	6442                	ld	s0,16(sp)
 6f0:	6161                	addi	sp,sp,80
 6f2:	8082                	ret

00000000000006f4 <printf>:

void
printf(const char *fmt, ...)
{
 6f4:	711d                	addi	sp,sp,-96
 6f6:	ec06                	sd	ra,24(sp)
 6f8:	e822                	sd	s0,16(sp)
 6fa:	1000                	addi	s0,sp,32
 6fc:	e40c                	sd	a1,8(s0)
 6fe:	e810                	sd	a2,16(s0)
 700:	ec14                	sd	a3,24(s0)
 702:	f018                	sd	a4,32(s0)
 704:	f41c                	sd	a5,40(s0)
 706:	03043823          	sd	a6,48(s0)
 70a:	03143c23          	sd	a7,56(s0)
  va_list ap;

  va_start(ap, fmt);
 70e:	00840613          	addi	a2,s0,8
 712:	fec43423          	sd	a2,-24(s0)
  vprintf(1, fmt, ap);
 716:	85aa                	mv	a1,a0
 718:	4505                	li	a0,1
 71a:	d2dff0ef          	jal	446 <vprintf>
}
 71e:	60e2                	ld	ra,24(sp)
 720:	6442                	ld	s0,16(sp)
 722:	6125                	addi	sp,sp,96
 724:	8082                	ret

0000000000000726 <free>:
static Header base;
static Header *freep;

void
free(void *ap)
{
 726:	1141                	addi	sp,sp,-16
 728:	e406                	sd	ra,8(sp)
 72a:	e022                	sd	s0,0(sp)
 72c:	0800                	addi	s0,sp,16
  Header *bp, *p;

  bp = (Header*)ap - 1;
 72e:	ff050693          	addi	a3,a0,-16
  for(p = freep; !(bp > p && bp < p->s.ptr); p = p->s.ptr)
 732:	00001797          	auipc	a5,0x1
 736:	8ce7b783          	ld	a5,-1842(a5) # 1000 <freep>
 73a:	a02d                	j	764 <free+0x3e>
    if(p >= p->s.ptr && (bp > p || bp < p->s.ptr))
      break;
  if(bp + bp->s.size == p->s.ptr){
    bp->s.size += p->s.ptr->s.size;
 73c:	4618                	lw	a4,8(a2)
 73e:	9f2d                	addw	a4,a4,a1
 740:	fee52c23          	sw	a4,-8(a0)
    bp->s.ptr = p->s.ptr->s.ptr;
 744:	6398                	ld	a4,0(a5)
 746:	6310                	ld	a2,0(a4)
 748:	a83d                	j	786 <free+0x60>
  } else
    bp->s.ptr = p->s.ptr;
  if(p + p->s.size == bp){
    p->s.size += bp->s.size;
 74a:	ff852703          	lw	a4,-8(a0)
 74e:	9f31                	addw	a4,a4,a2
 750:	c798                	sw	a4,8(a5)
    p->s.ptr = bp->s.ptr;
 752:	ff053683          	ld	a3,-16(a0)
 756:	a091                	j	79a <free+0x74>
    if(p >= p->s.ptr && (bp > p || bp < p->s.ptr))
 758:	6398                	ld	a4,0(a5)
 75a:	00e7e463          	bltu	a5,a4,762 <free+0x3c>
 75e:	00e6ea63          	bltu	a3,a4,772 <free+0x4c>
{
 762:	87ba                	mv	a5,a4
  for(p = freep; !(bp > p && bp < p->s.ptr); p = p->s.ptr)
 764:	fed7fae3          	bgeu	a5,a3,758 <free+0x32>
 768:	6398                	ld	a4,0(a5)
 76a:	00e6e463          	bltu	a3,a4,772 <free+0x4c>
    if(p >= p->s.ptr && (bp > p || bp < p->s.ptr))
 76e:	fee7eae3          	bltu	a5,a4,762 <free+0x3c>
  if(bp + bp->s.size == p->s.ptr){
 772:	ff852583          	lw	a1,-8(a0)
 776:	6390                	ld	a2,0(a5)
 778:	02059813          	slli	a6,a1,0x20
 77c:	01c85713          	srli	a4,a6,0x1c
 780:	9736                	add	a4,a4,a3
 782:	fae60de3          	beq	a2,a4,73c <free+0x16>
    bp->s.ptr = p->s.ptr->s.ptr;
 786:	fec53823          	sd	a2,-16(a0)
  if(p + p->s.size == bp){
 78a:	4790                	lw	a2,8(a5)
 78c:	02061593          	slli	a1,a2,0x20
 790:	01c5d713          	srli	a4,a1,0x1c
 794:	973e                	add	a4,a4,a5
 796:	fae68ae3          	beq	a3,a4,74a <free+0x24>
    p->s.ptr = bp->s.ptr;
 79a:	e394                	sd	a3,0(a5)
  } else
    p->s.ptr = bp;
  freep = p;
 79c:	00001717          	auipc	a4,0x1
 7a0:	86f73223          	sd	a5,-1948(a4) # 1000 <freep>
}
 7a4:	60a2                	ld	ra,8(sp)
 7a6:	6402                	ld	s0,0(sp)
 7a8:	0141                	addi	sp,sp,16
 7aa:	8082                	ret

00000000000007ac <malloc>:
  return freep;
}

void*
malloc(uint nbytes)
{
 7ac:	7139                	addi	sp,sp,-64
 7ae:	fc06                	sd	ra,56(sp)
 7b0:	f822                	sd	s0,48(sp)
 7b2:	f04a                	sd	s2,32(sp)
 7b4:	ec4e                	sd	s3,24(sp)
 7b6:	0080                	addi	s0,sp,64
  Header *p, *prevp;
  uint nunits;

  nunits = (nbytes + sizeof(Header) - 1)/sizeof(Header) + 1;
 7b8:	02051993          	slli	s3,a0,0x20
 7bc:	0209d993          	srli	s3,s3,0x20
 7c0:	09bd                	addi	s3,s3,15
 7c2:	0049d993          	srli	s3,s3,0x4
 7c6:	2985                	addiw	s3,s3,1
 7c8:	894e                	mv	s2,s3
  if((prevp = freep) == 0){
 7ca:	00001517          	auipc	a0,0x1
 7ce:	83653503          	ld	a0,-1994(a0) # 1000 <freep>
 7d2:	c905                	beqz	a0,802 <malloc+0x56>
    base.s.ptr = freep = prevp = &base;
    base.s.size = 0;
  }
  for(p = prevp->s.ptr; ; prevp = p, p = p->s.ptr){
 7d4:	611c                	ld	a5,0(a0)
    if(p->s.size >= nunits){
 7d6:	4798                	lw	a4,8(a5)
 7d8:	09377663          	bgeu	a4,s3,864 <malloc+0xb8>
 7dc:	f426                	sd	s1,40(sp)
 7de:	e852                	sd	s4,16(sp)
 7e0:	e456                	sd	s5,8(sp)
 7e2:	e05a                	sd	s6,0(sp)
  if(nu < 4096)
 7e4:	8a4e                	mv	s4,s3
 7e6:	6705                	lui	a4,0x1
 7e8:	00e9f363          	bgeu	s3,a4,7ee <malloc+0x42>
 7ec:	6a05                	lui	s4,0x1
 7ee:	000a0b1b          	sext.w	s6,s4
  p = sbrk(nu * sizeof(Header));
 7f2:	004a1a1b          	slliw	s4,s4,0x4
        p->s.size = nunits;
      }
      freep = prevp;
      return (void*)(p + 1);
    }
    if(p == freep)
 7f6:	00001497          	auipc	s1,0x1
 7fa:	80a48493          	addi	s1,s1,-2038 # 1000 <freep>
  if(p == (char*)-1)
 7fe:	5afd                	li	s5,-1
 800:	a83d                	j	83e <malloc+0x92>
 802:	f426                	sd	s1,40(sp)
 804:	e852                	sd	s4,16(sp)
 806:	e456                	sd	s5,8(sp)
 808:	e05a                	sd	s6,0(sp)
    base.s.ptr = freep = prevp = &base;
 80a:	00001797          	auipc	a5,0x1
 80e:	80678793          	addi	a5,a5,-2042 # 1010 <base>
 812:	00000717          	auipc	a4,0x0
 816:	7ef73723          	sd	a5,2030(a4) # 1000 <freep>
 81a:	e39c                	sd	a5,0(a5)
    base.s.size = 0;
 81c:	0007a423          	sw	zero,8(a5)
    if(p->s.size >= nunits){
 820:	b7d1                	j	7e4 <malloc+0x38>
        prevp->s.ptr = p->s.ptr;
 822:	6398                	ld	a4,0(a5)
 824:	e118                	sd	a4,0(a0)
 826:	a899                	j	87c <malloc+0xd0>
  hp->s.size = nu;
 828:	01652423          	sw	s6,8(a0)
  free((void*)(hp + 1));
 82c:	0541                	addi	a0,a0,16
 82e:	ef9ff0ef          	jal	726 <free>
  return freep;
 832:	6088                	ld	a0,0(s1)
      if((p = morecore(nunits)) == 0)
 834:	c125                	beqz	a0,894 <malloc+0xe8>
  for(p = prevp->s.ptr; ; prevp = p, p = p->s.ptr){
 836:	611c                	ld	a5,0(a0)
    if(p->s.size >= nunits){
 838:	4798                	lw	a4,8(a5)
 83a:	03277163          	bgeu	a4,s2,85c <malloc+0xb0>
    if(p == freep)
 83e:	6098                	ld	a4,0(s1)
 840:	853e                	mv	a0,a5
 842:	fef71ae3          	bne	a4,a5,836 <malloc+0x8a>
  p = sbrk(nu * sizeof(Header));
 846:	8552                	mv	a0,s4
 848:	b33ff0ef          	jal	37a <sbrk>
  if(p == (char*)-1)
 84c:	fd551ee3          	bne	a0,s5,828 <malloc+0x7c>
        return 0;
 850:	4501                	li	a0,0
 852:	74a2                	ld	s1,40(sp)
 854:	6a42                	ld	s4,16(sp)
 856:	6aa2                	ld	s5,8(sp)
 858:	6b02                	ld	s6,0(sp)
 85a:	a03d                	j	888 <malloc+0xdc>
 85c:	74a2                	ld	s1,40(sp)
 85e:	6a42                	ld	s4,16(sp)
 860:	6aa2                	ld	s5,8(sp)
 862:	6b02                	ld	s6,0(sp)
      if(p->s.size == nunits)
 864:	fae90fe3          	beq	s2,a4,822 <malloc+0x76>
        p->s.size -= nunits;
 868:	4137073b          	subw	a4,a4,s3
 86c:	c798                	sw	a4,8(a5)
        p += p->s.size;
 86e:	02071693          	slli	a3,a4,0x20
 872:	01c6d713          	srli	a4,a3,0x1c
 876:	97ba                	add	a5,a5,a4
        p->s.size = nunits;
 878:	0137a423          	sw	s3,8(a5)
      freep = prevp;
 87c:	00000717          	auipc	a4,0x0
 880:	78a73223          	sd	a0,1924(a4) # 1000 <freep>
      return (void*)(p + 1);
 884:	01078513          	addi	a0,a5,16
  }
}
 888:	70e2                	ld	ra,56(sp)
 88a:	7442                	ld	s0,48(sp)
 88c:	7902                	ld	s2,32(sp)
 88e:	69e2                	ld	s3,24(sp)
 890:	6121                	addi	sp,sp,64
 892:	8082                	ret
 894:	74a2                	ld	s1,40(sp)
 896:	6a42                	ld	s4,16(sp)
 898:	6aa2                	ld	s5,8(sp)
 89a:	6b02                	ld	s6,0(sp)
 89c:	b7f5                	j	888 <malloc+0xdc>
