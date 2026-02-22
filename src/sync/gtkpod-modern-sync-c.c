#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include <glib.h>
#include <gpod/itdb.h>

static const char* get_arg(int argc, char** argv, const char* key) {
    for (int i = 1; i + 1 < argc; i++) {
        if (strcmp(argv[i], key) == 0) return argv[i + 1];
    }
    return NULL;
}

static int file_exists(const char* path) {
    return g_file_test(path, G_FILE_TEST_EXISTS);
}

int main(int argc, char** argv) {
    // line-buffer stdout so Rust UI sees progress immediately
    setvbuf(stdout, NULL, _IOLBF, 0);

    const char* mount_path = get_arg(argc, argv, "--mount");
    const char* sqlite_path = get_arg(argc, argv, "--sqlite");

    if (!mount_path || !sqlite_path) {
        printf("ERROR usage: %s --mount <ipod_mount> --sqlite <library.db>\n", argv[0]);
        return 2;
    }

    printf("PROGRESS starting iPod sync...\n");
    printf("PROGRESS mount=%s\n", mount_path);
    printf("PROGRESS sqlite=%s\n", sqlite_path);

    // For iPod touch (AFC2), passing the folder that CONTAINS iPod_Control works.
    // You said you mount: /home/.../ipod/var/mobile/Media
    // That should contain: iPod_Control/iTunes/iTunesDB
    char *itunesdb = g_build_filename(mount_path, "iTunes_Control", "iTunes", "iTunesDB", NULL);
    char *itunesdb_shm = g_build_filename(mount_path, "iTunes_Control", "iTunes", "iTunesDB.shm", NULL);

    if (!file_exists(itunesdb) && !file_exists(itunesdb_shm)) {
        printf("ERROR iTunesDB not found under: %s/iTunes_Control/iTunes/\n", mount_path);
        printf("ERROR check your mount path points to a folder containing iTunes_Control\n");
        g_free(itunesdb);
        g_free(itunesdb_shm);
        return 3;
    }

    printf("PROGRESS found iTunesDB\n");

    GError *err = NULL;
    Itdb_iTunesDB *itdb = itdb_parse(mount_path, &err);
    if (!itdb) {
        printf("ERROR libgpod itdb_parse failed: %s\n", err ? err->message : "unknown");
        if (err) g_error_free(err);
        g_free(itunesdb);
        g_free(itunesdb_shm);
        return 4;
    }

    // Count tracks (libgpod stores them in a GList)
    int count = 0;
    for (GList *l = itdb->tracks; l != NULL; l = l->next) {
        count++;
    }

    printf("PROGRESS iPod DB opened. tracks=%d\n", count);

    // Clean up
    itdb_free(itdb);
    g_free(itunesdb);
    g_free(itunesdb_shm);

    printf("DONE\n");
    return 0;
}
