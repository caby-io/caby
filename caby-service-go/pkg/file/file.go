package file

import (
	"caby-service/pkg/pretty"
	"fmt"
	"io/fs"
	"log/slog"
	"math"
	"os"
	"syscall"
	"time"
)

type File struct {
	Name             string    `json:"name"`
	Size             int64     `json:"size"`
	PrettySize       string    `json:"prettySize"`
	CreatedAt        time.Time `json:"createdAt"`
	PrettyCreatedAt  string    `json:"prettyCreatedAt"`
	ModifiedAt       time.Time `json:"modifiedAt"`
	PrettyModifiedAt string    `json:"prettyModifiedAt"`
	// FileInfo   fs.FileInfo `json:"-"`
	// Hash       []byte      `json:"hash"`
}

// Credit: https://gist.github.com/maxmcd
func prettyByteSize(b int) string {
	bf := float64(b)
	// todo: removed 'i', consider whether this is accurate.
	for _, unit := range []string{"", "K", "M", "G", "T", "P", "E", "Z"} {
		if math.Abs(bf) < 1024.0 {
			return fmt.Sprintf("%3.1f %sB", bf, unit)
		}
		bf /= 1024.0
	}
	return fmt.Sprintf("%.1fYB", bf)
}

// todo: handle each OS and have a fallback
// e.g. use fileinfo.ModTime
func statTimes(path string) (atime, mtime, ctime time.Time, err error) {
	fi, err := os.Stat(path)
	if err != nil {
		return
	}
	mtime = fi.ModTime()
	stat := fi.Sys().(*syscall.Stat_t)
	atime = time.Unix(int64(stat.Atim.Sec), int64(stat.Atim.Nsec))
	ctime = time.Unix(int64(stat.Ctim.Sec), int64(stat.Ctim.Nsec))
	return
}

// Temp?
func NewFile(path string, fileinfo fs.FileInfo) File {
	// TEMP
	_, mt, ct, err := statTimes(path)
	if err != nil {
		// todo: handle
		slog.Error("couldnt get file time info", "statTimes.err", err)
	}

	return File{
		Name:             fileinfo.Name(),
		Size:             fileinfo.Size(),
		PrettySize:       prettyByteSize(int(fileinfo.Size())),
		CreatedAt:        ct,
		PrettyCreatedAt:  pretty.Date(ct),
		ModifiedAt:       mt,
		PrettyModifiedAt: pretty.Date(mt),
	}
}
