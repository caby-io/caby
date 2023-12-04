package file

import (
	"io/fs"
	"log/slog"
	"time"
)

type Dir struct {
	Name string `json:"name"`
	// TODO
	// Size       int64     `json:"size"`
	// PrettySize string    `json:"prettySize"`
	CreatedAt  time.Time `json:"createdAt"`
	ModifiedAt time.Time `json:"modifiedAt"`
}

func NewDir(fileinfo fs.FileInfo) Dir {
	// TEMP
	_, mt, ct, err := statTimes(fileinfo.Name())
	if err != nil {
		slog.Error("couldnt get file time info", "statTimes.err", err)
		return Dir{
			Name:       fileinfo.Name(),
			CreatedAt:  fileinfo.ModTime(),
			ModifiedAt: fileinfo.ModTime(),
		}
	}

	return Dir{
		Name:       fileinfo.Name(),
		CreatedAt:  ct,
		ModifiedAt: mt,
	}
}
