#include <stdlib.h>

int main(void) {
  int *p = malloc(sizeof(int));
  free(p);
  return 0;
}
