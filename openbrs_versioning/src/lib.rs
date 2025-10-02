/*
* Plan and design:
* To design a versioning system, I'll use three types of objects, just like Git:
*  1. Blob: represents the content of a file (after optional compression + encryption).
       * Stored by hash (e.g., SHA-256).
       * Identical content -> identical blob -> deduplication for free.
   2. Tree: represents a directory.
       * Maps file/directory names -> blob IDs or sub-tree IDs.
       * Basically the "snapshot of a folder."
   3. Commit: represents a backup session.
       * Points to a root tree.
       * Contains metadata (timestamp, backup type, salts, nonces, et cetera.).
       * May reference a parent commit (for differential/incremental backups).
*/
