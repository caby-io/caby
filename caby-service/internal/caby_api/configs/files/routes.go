package files

import (
	"caby-service/pkg/config"
	"log/slog"
	"net/http"
	"path/filepath"
	"strings"
	"sync"

	"github.com/go-chi/chi/v5"
	"github.com/karrick/godirwalk"
)

func HandleRoutes(cfg config.Config) chi.Router {
	r := chi.NewRouter()

	r.Get("/*", handleFileFolder(cfg))

	return r
}

// TEMP
func handleFileFolder(cfg config.Config) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		path := chi.URLParam(r, "*")

		results := []string{}
		mu := sync.Mutex{}

		godirwalk.Walk(filepath.Join(cfg.DataPath, path), &godirwalk.Options{
			FollowSymbolicLinks: false,
			Unsorted:            true,
			Callback: func(osPathname string, de *godirwalk.Dirent) error {
				// do not go deep for this particular endpoint
				// if de.IsDir() {
				// 	return nil
				// }

				mu.Lock()
				defer mu.Unlock()

				results = append(results, osPathname)
				return nil
			},
			ErrorCallback: func(osPathname string, err error) godirwalk.ErrorAction {
				return godirwalk.SkipNode
			},
		})

		slog.Info(filepath.Join(cfg.DataPath, path))
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(strings.Join(results, "\n")))
	}
}
