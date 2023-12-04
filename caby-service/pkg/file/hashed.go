package file

import "io/fs"

type HashedEntry struct {
	Entry fs.DirEntry
	Hash  []byte
}
