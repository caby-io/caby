package file

import (
	"caby-service/pkg/pretty"
	"io/fs"
	"log/slog"
	"strings"
	"time"
)

type Dir struct {
	Name string `json:"name"`
	// TODO
	// Size       int64     `json:"size"`
	// PrettySize string    `json:"prettySize"`
	CreatedAt        time.Time `json:"createdAt"`
	PrettyCreatedAt  string    `json:"prettyCreatedAt"`
	ModifiedAt       time.Time `json:"modifiedAt"`
	PrettyModifiedAt string    `json:"prettyModifiedAt"`
}

func NewDir(path string, entry fs.DirEntry) Dir {
	// TEMP
	_, mt, ct, err := statTimes(path)
	if err != nil {
		// todo: handle
		slog.Error("couldnt get file time info", "statTimes.err", err)
	}

	return Dir{
		Name:             entry.Name(),
		CreatedAt:        ct,
		PrettyCreatedAt:  pretty.Date(ct),
		ModifiedAt:       mt,
		PrettyModifiedAt: pretty.Date(mt),
	}
}

// SanitizePath outputs a predictible path regardless of the user input.
// We want: /folder/folder_or_file. For the root we want: "/".
func SanitizePath(path string) string {
	split := strings.Split(strings.TrimSpace(path), "/")

	final := []string{}
	for _, s := range split {
		if s != "" {
			final = append(final, s)
		}
	}

	return strings.Join(final, "/")
}
