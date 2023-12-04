package files

import (
	"caby-service/pkg/config"
	"caby-service/pkg/file"
	"caby-service/pkg/jsend"
	"log/slog"
	"net/http"
	"os"
	"path/filepath"
	"unsafe"

	"github.com/go-chi/chi/v5"
)

type HandleFilesResp struct {
	Dirs  []file.Dir  `json:"dirs"`
	Files []file.File `json:"files"`
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

		resp := HandleFilesResp{}
		for _, e := range entries {
			i, err := e.Info()
			if err != nil {
				slog.Error("couldn't get entry info", "entry", filepath.Join(fullPath, e.Name()))
			}

			if !e.IsDir() {
				resp.Files = append(resp.Files, file.NewFile(i))
				continue
			}
			resp.Dirs = append(resp.Dirs, file.NewDir(i))
		}

		// TEMP PROFILE
		totalSize := uintptr(0)
		for _, f := range resp.Files {
			totalSize += unsafe.Sizeof(f)
		}

		slog.Debug("slice size", "bytes", unsafe.Sizeof(resp))
		slog.Debug("contents size", "bytes", totalSize)

		slog.Info(filepath.Join(cfg.DataPath, path))
		jsend.New().Ok().Data(resp).Write(w)
	}
}
