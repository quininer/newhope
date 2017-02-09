#include <stdlib.h>
#include "newhope.h"
#include "poly.h"

poly *newhope_keygen_poly(unsigned char *send) {
	poly *ska = calloc(sizeof(poly), 1);
	newhope_keygen(send, ska);
	return ska;
}
