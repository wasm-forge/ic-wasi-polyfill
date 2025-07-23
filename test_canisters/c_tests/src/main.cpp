#include <iostream>
#include <string>
#include <fstream>
#include <unistd.h>
#include <sys/stat.h>

#define __IMPORT(module, name) __attribute__((__import_module__(#module), __import_name__(#name)))
#define __EXPORT(name) __attribute__((__export_name__(#name)))

// Initialize the WASI polyfill library first.
extern "C" void raw_init(char* p, std::size_t len) __IMPORT(polyfill, raw_init);
class WasiPolyfill{
  public:
    WasiPolyfill(){
        raw_init(nullptr, 0);
    }
} __wasi_polyfill;

extern "C" void ic0_debug_print(const char *str, std::size_t len) __IMPORT(ic0, debug_print);
extern "C" int  ic0_msg_arg_data_size() __IMPORT(ic0, msg_arg_data_size);
extern "C" void ic0_msg_arg_data_copy(char * buf, std::size_t offset, std::size_t length) __IMPORT(ic0, msg_arg_data_copy);
extern "C" void ic0_msg_reply() __IMPORT(ic0, msg_reply);
extern "C" void ic0_msg_reply_data_append(const char * buf, std::size_t length) __IMPORT(ic0, msg_reply_data_append);

// some static variable
const std::vector<std::string> s_msg ({"Hello, ", "world"});

extern "C" __EXPORT(canister_query greet) __attribute__((noinline)) void greet()  {

    int n = ic0_msg_arg_data_size();
    char buf[n];

    ic0_msg_arg_data_copy(buf, 0, n);
    
    // work with text
    std::string s(buf);
    
    std::string content = s_msg[0] + s;

    // do some file operations
    std::ofstream ofile("content.txt");
    ofile << "File content: " << content;
    ofile.close();

    std::string line;
    std::ifstream ifile("content.txt");
    getline (ifile,line);
    ifile.close();

    ic0_msg_reply_data_append(line.c_str(), line.length());
    ic0_msg_reply();
}

extern "C" __EXPORT(canister_query test_access) __attribute__((noinline)) void test_access()  {
    char buf[100];

    ic0_debug_print("Testing access", 14);

    mkdir("./tmp", 0777);
    mkdir("./tmp/test", 0777);

    int r;

    r = access("./tmp", F_OK);
    sprintf(buf, "access F_OK = %d", F_OK);
    ic0_debug_print(buf, strlen(buf));

    r = access("./tmp", R_OK);
    sprintf(buf, "access R_OK = %d", R_OK);
    ic0_debug_print(buf, strlen(buf));

    r = access("./tmp", W_OK);
    sprintf(buf, "access W_OK = %d", W_OK);
    ic0_debug_print(buf, strlen(buf));


    ic0_debug_print("Done testing!", 13);

    ic0_msg_reply();
}


void deb_print(const char *str) {
    ic0_debug_print(str, strlen(str));
}

extern "C" __EXPORT(canister_query test_stat) __attribute__((noinline)) void test_stat()  {
    char buf[100];

    deb_print("\n\n\n\n\n\nTesting access\n\n");

    // either test file or folder
    if (1) {
        std::ofstream ofs("/tmp", std::ios::binary | std::ios::out);
        ofs.seekp(1011);
        ofs.write("abcabcabc", 1);
    } else {
        mkdir("/tmp", 0777);
        mkdir("/tmp/test", 0777);
    }

    struct stat sb;
    int r = stat("./tmp", &sb);

    sprintf(buf, "stat returns %d\n", r);
    deb_print(buf);

    ////////////////

    switch (sb.st_mode & S_IFMT) {
        case S_IFBLK:  deb_print("block device");            break;
        case S_IFCHR:  deb_print("character device");        break;
        case S_IFDIR:  deb_print("directory");               break;
        case S_IFIFO:  deb_print("FIFO/pipe");               break;
        case S_IFLNK:  deb_print("symlink");                 break;
        case S_IFREG:  deb_print("regular file");            break;
        case S_IFSOCK: deb_print("socket");                  break;
        default:       deb_print("unknown?");                break;
    }

    sprintf(buf, "I-node number:            %ju\n", (uintmax_t) sb.st_ino);
    deb_print(buf);
    
    sprintf(buf, "Mode %jo (octal)\n", (uintmax_t) sb.st_mode);
    deb_print(buf);

    sprintf(buf, "Link count %jd \n", (uintmax_t) sb.st_nlink);
    deb_print(buf);

    sprintf(buf, "Ownership:                UID=%ju   GID=%ju\n", (uintmax_t) sb.st_uid, (uintmax_t) sb.st_gid);
    deb_print(buf);

    sprintf(buf, "Preferred I/O block size: %jd bytes\n", (intmax_t) sb.st_blksize);
    deb_print(buf);

    sprintf(buf, "File size:                %jd bytes\n", (intmax_t) sb.st_size);
    deb_print(buf);

    sprintf(buf, "Blocks allocated:         %jd\n", (intmax_t) sb.st_blocks);
    deb_print(buf);

    deb_print("Done testing!");

    ic0_msg_reply();
}
