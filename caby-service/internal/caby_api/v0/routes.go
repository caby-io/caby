package v0_routes

import (
	"caby-service/internal/caby_api/v0/files"
	"caby-service/pkg/config"

	"github.com/go-chi/chi/v5"
)

func GetRoutes(cfg config.Config) chi.Router {
	r := chi.NewRouter()

	// todo: consolidate these?
	r.Get("/files", files.HandleFiles(cfg))
	r.Get("/files/*", files.HandleFiles(cfg))

	return r
}
