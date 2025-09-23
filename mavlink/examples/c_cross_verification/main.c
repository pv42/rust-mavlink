#include "../../../c_library_v2/all/mavlink.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <assert.h>

int main() {
    mavlink_status_t status;
    mavlink_message_t msg;
    int chan = MAVLINK_COMM_0;

    FILE *fd;

    // Open a file in read mode
    fd = fopen("filename.bin", "rb"); 

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
#include "c_msg_asserts.c"
            parsed = true;
            break;
        }
    }
    assert(parsed);
    return 0;
}