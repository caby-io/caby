package fs

import (
	"caby-service/pkg/file"
	"log/slog"
	"os"
	"path/filepath"
)

type BadPathErr struct {
	Path string
}

func (e BadPathErr) Error() string {
	return "could not read path: " + e.Path
}

// todo: accept some config context
// GetContents fetches the files and folders at the path
func GetContents(path string) ([]file.Dir, []file.File, error) {
	entries, err := os.ReadDir(path)
	if err != nil {
		return nil, nil, BadPathErr{path}
	}

	dirs := []file.Dir{}
	files := []file.File{}
	for _, e := range entries {
		i, err := e.Info()
		if err != nil {
			// todo: replace with errr
			slog.Error("couldn't get entry info", "entry", filepath.Join(path, e.Name()))
			continue
		}

		if e.IsDir() {
			dirs = append(dirs, file.NewDir(filepath.Join(path, e.Name()), e))
			continue
		}

		files = append(files, file.NewFile(path, i))
	}

	return dirs, files, nil
}
