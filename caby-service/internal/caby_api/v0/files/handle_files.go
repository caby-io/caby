package files

import (
	"caby-service/pkg/config"
	"caby-service/pkg/jsend"
	"io/fs"
	"log/slog"
	"net/http"
	"os"
	"path/filepath"
	"unsafe"

	"github.com/go-chi/chi/v5"
)

// TEMP
type DirEntry struct {
	Name  string
	IsDir bool
	Type  string
}

// TEMP
func fromFs(d fs.DirEntry) DirEntry {
	return DirEntry{
		Name:  d.Name(),
		IsDir: d.IsDir(),
		Type:  d.Type().String(),
	}
}

func HandleFiles(cfg config.Config) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		path := chi.URLParam(r, "*")

		fullPath := filepath.Join(cfg.DataPath, path)
		entries, err := os.ReadDir(fullPath)
		if err != nil {
			slog.Error("couldn't read path", "path", fullPath, "os_err", err)
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		results := []DirEntry{}
		for _, e := range entries {
			results = append(results, fromFs(e))
		}

		slog.Debug("profile", "bytes", unsafe.Sizeof(results))
		// slog.Debug("profile", "bytes", unsafe.Offsetof(results[0]))

		// fmt.Println(unsafe.Sizeof(results[0]))
		// 	"offset=", unsafe.Offsetof(t.I))

		slog.Info(filepath.Join(cfg.DataPath, path))
		jsend.New().Ok().Data(results).Write(w)
	}
}
