package main

import (
	"caby-service/internal/caby_api/configs"
	"caby-service/internal/caby_api/configs/files"
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
)

func main() {
	cfg := configs.LoadConfig()

	// todo: temp
	r := chi.NewRouter()
	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)

	r.Mount("/files", files.HandleRoutes(cfg))

	http.ListenAndServe(":8080", r)
}
