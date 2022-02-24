#include <hycron/stdbool.h>
#include <string.h>
#include <unistd.h>
int foo = 1;
char *bar = "str";
bool baz = true;
int USER_DEFINED_qux(int lhs, int rhs)
{
    return lhs + rhs;
}
int main(int ARGC, char **ARGV)
{
    char *msg = "Hello, World!";
    write(1, msg, strlen(msg));
    return 0;
}
