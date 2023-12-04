package main

import (
	"caby-service/internal/caby_api/configs"
	v0_routes "caby-service/internal/caby_api/v0"
	"log/slog"
	"net/http"
	"os"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/cors"
)

// temp
var programLevel = new(slog.LevelVar)

func main() {
	// temp
	h := slog.NewJSONHandler(os.Stderr, &slog.HandlerOptions{Level: programLevel})
	slog.SetDefault(slog.New(h))
	programLevel.Set(slog.LevelDebug)

	cfg := configs.LoadConfig()

	// todo: temp
	r := chi.NewRouter()
	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)

	r.Use(cors.Handler(cors.Options{AllowedOrigins: []string{"*"}}))

	r.Mount("/v0", v0_routes.GetRoutes(cfg))

	http.ListenAndServe(":8080", r)
}
