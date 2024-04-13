package files

import (
	"caby-service/pkg/config"
	"caby-service/pkg/jsend"
	"io"
	"log/slog"
	"net/http"
	"os"
	"path/filepath"

	"github.com/davecgh/go-spew/spew"
)

func HandleUploadFiles(cfg config.Config) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		err := r.ParseMultipartForm(32 << 20)
		if err != nil {
			slog.Error("could not parse multipart form", "err", err)
			jsend.New().ServerError().Write(w)
			return
		}

		f, h, err := r.FormFile("file")
		spew.Dump(h)
		if err != nil {
			slog.Error("could not get file info/data", "err", err)
			jsend.New().ServerError().Write(w)
			return
		}

		dir := filepath.Join(".", "files")
		err = os.MkdirAll(dir, os.ModePerm)
		if err != nil {
			slog.Error("could not create destination dir", "err", err)
			jsend.New().ServerError().Write(w)
			return
		}
		path := filepath.Join(dir, r.Form.Get("name"))

		file, err := os.OpenFile(path, os.O_WRONLY|os.O_CREATE, os.ModePerm)
		if err != nil {
			slog.Error("could not open file for writing", "err", err)
			jsend.New().ServerError().Write(w)
			return
		}
		defer file.Close()

		_, err = io.Copy(file, f)
		if err != nil {
			slog.Error("could not write to file", "err", err)
			jsend.New().ServerError().Write(w)
			return
		}

		jsend.New().Ok().Write(w)
		// return n + filepath.Ext(h.Filename)
	}
}
