/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <firestorm.h>

char hello[] = "Hello World!\r\n";

CB_TYPE nop(int x, int y, int z, void *ud) { return ASYNC; }

void main() {
  putnstr_async(hello, sizeof(hello), nop, NULL);
}

