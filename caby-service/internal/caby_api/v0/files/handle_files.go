package files

import (
	"caby-service/internal/caby_api/fs"
	"caby-service/pkg/config"
	"caby-service/pkg/file"
	"caby-service/pkg/jsend"
	"log/slog"
	"net/http"
	"net/url"
	"path/filepath"

	"github.com/go-chi/chi/v5"
)

type HandleFilesResp struct {
	ParentPath  *string     `json:"parentPath"`
	CurrentPath string      `json:"currentPath"`
	Dirs        []file.Dir  `json:"dirs"`
	Files       []file.File `json:"files"`
}

func HandleFiles(cfg config.Config) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		escapedPath, err := url.PathUnescape(chi.URLParam(r, "*"))
		if err != nil {
			slog.Error("couldn't unescape path", "path", chi.URLParam(r, "*"), "error", err)
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		sanitizedPath := file.SanitizePath(escapedPath)

		dirs, files, err := fs.GetContents(filepath.Join(cfg.DataPath, sanitizedPath))
		if err != nil {
			slog.Error("couldn't read path", "path", filepath.Join(cfg.DataPath, sanitizedPath), "error", err)
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		var parentPath *string = nil
		// We are in the root dir
		if sanitizedPath != "" {
			p, _ := filepath.Split(sanitizedPath)
			parentPath = &p
		}

		resp := HandleFilesResp{
			parentPath, sanitizedPath, dirs, files,
		}

		jsend.New().Ok().Data(resp).Write(w)
	}
}
