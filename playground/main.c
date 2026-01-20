
void integer_literals() {
    int dec = 42;
    unsigned int u_dec = 42U;
    long long_dec = 42L;
    unsigned long u_long_dec = 42UL;
    long long long_long_dec = 42LL;
    unsigned long long u_long_long_dec = 42ULL;

    int oct = 052;
    unsigned int u_oct = 052U;
    long long_oct = 052L;
    unsigned long u_long_oct = 052UL;
    long long long_long_oct = 052LL;
    unsigned long long u_long_long_oct = 052ULL;

    int hex = 0x2A;
    unsigned int u_hex = 0x2AU;
    long long_hex = 0x2AL;
    unsigned long u_long_hex = 0x2AUL;
    long long long_long_hex = 0x2ALL;
    unsigned long long u_long_long_hex = 0x2AULL;

    int bin = 0b101010;
    unsigned int u_bin = 0b101010U;
    long long_bin = 0b101010L;
    unsigned long u_long_bin = 0b101010UL;
    long long long_long_bin = 0b101010LL;
    unsigned long long u_long_long_bin = 0b101010ULL;
}

void float_literals() {
    float f1 = 3.14f;
    double d1 = 3.14;
    long double ld1 = 3.14L;

    float f2 = 2.5e3f;
    double d2 = 2.5e3;
    long double ld2 = 2.5e3L;

    float f3 = 1.0E-4f;
    double d3 = 1.0E-4;
    long double ld3 = 1.0E-4L;
}

int main() {
    integer_literals();

    float_literals();

    return 0;
}