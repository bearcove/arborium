; test source file for assembling to ELF
; build with:
;    nasm -f elf elftest.asm
;    gcc -o elftest elftest.c elftest.o

	  BITS 32
	  GLOBAL lrotate
	  GLOBAL greet
	  GLOBAL asmstr
	  GLOBAL textptr
	  GLOBAL selfptr
	  GLOBAL integer
	  EXTERN printf
	  COMMON commvar 4

	  SECTION .text

; prototype: long lrotate(long x, int num);
lrotate:
	  push ebp
	  mov ebp,esp
	  mov eax,[ebp+8]
	  mov ecx,[ebp+12]
.label	  rol eax,1
	  loop .label
	  mov esp,ebp
	  pop ebp
	  ret

; prototype: void greet(void);
greet	  mov eax,[integer]
	  inc eax
	  mov [localint],eax
	  push dword [commvar]
	  mov eax,[localptr]
	  push dword [eax]
	  push dword [integer]
	  push dword printfstr
	  call printf
	  add esp,16
	  ret

	  SECTION .data

; a string
asmstr	  db 'hello, world', 0

; a string for Printf
printfstr db "integer==%d, localint==%d, commvar=%d"
	  db 10, 0

; some pointers
localptr  dd localint
textptr	  dd greet
selfptr	  dd selfptr

	  SECTION .bss

; an integer
integer	  resd 1

; a local integer
localint  resd 1

	  SECTION .rodata
readonly  dd readonly
