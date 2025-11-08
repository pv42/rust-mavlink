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

#include "c_msg_asserts.c"

void check_msg_file(uint32_t msg_id) {
    printf("Preperaing msg id %d\n", msg_id);
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
    // Open a file in read mode
    for(int i = 0; i < 8000; i++) {
        check_msg_file(i);
    }
    return 0;
}