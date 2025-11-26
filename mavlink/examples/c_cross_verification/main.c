#include "../../../c_library_v2/all/mavlink.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <assert.h>
#include <string.h>

#define typename(x) _Generic((x), \
    int:     "int", \
    float:   "float", \
    char:    "char", \
    default: "other")

void assert_str_eq(const char* value, const char* expected) {
    if (strcmp(value, expected) != 0) {
        printf("Assertion failed: expected '%s' but got '%s'\n", expected, value);
        printf("Bytes:");
        const char* ep = expected;
        const char* vp = value;
        while(*ep != 0) {
            printf("%d ", *ep);
            ep ++;
        }
        printf("vs ");
        while(*vp != 0) {
            printf("%d ", *vp);
            vp ++;
        }
        printf(" \n");
        abort();
    }
}

#include "c_msg_asserts.c"

void check_msg_file(uint32_t msg_id) {
    mavlink_status_t status;
    mavlink_message_t msg;
    int chan = MAVLINK_COMM_0;
    FILE *fd;

    char filename[64];
    sprintf(filename, "messages/msg_%u.bin", msg_id);

    fd = fopen(filename, "rb"); 

    if (!fd) {
        return;
    }
    
    printf("Preperaing msg id %d\n", msg_id);

    char buf[BUFSIZ+1];
    long n = 0;
    while(true) {
        int ch = fgetc(fd);
        if(ch != EOF) {
            buf[n] = ch;
            n++;
        } else {
            break;
        }
    };
    printf("Read %ld bytes \n",n);    
    bool parsed = false;
    for(int i = 0; i < n; i++) {
        if (mavlink_parse_char(chan, buf[i], &msg, &status)) {
            printf("MSG_ID: %d\n", msg.msgid);
            check_msg(msg_id, &msg);
            parsed = true;
            break;
        }
    }
    assert(parsed);
}

int main() {
    for(int i = 0; i < 60100; i++) {
        check_msg_file(i);
    }
    return 0;
}