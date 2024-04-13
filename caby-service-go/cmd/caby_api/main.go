package main

import (
	"caby-service/internal/caby_api/configs"
	v0_routes "caby-service/internal/caby_api/v0"
	"log/slog"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/cors"
	"github.com/lmittmann/tint"
	slogchi "github.com/samber/slog-chi"
)

// temp
var programLevel = new(slog.LevelVar)

func main() {
	logger := slog.New(
		tint.NewHandler(os.Stderr, &tint.Options{
			Level:      programLevel,
			TimeFormat: time.Kitchen,
		}),
	)
	slog.SetDefault(logger)
	programLevel.Set(slog.LevelDebug)

	cfg := configs.LoadConfig()

	// todo: temp
	r := chi.NewRouter()
	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	// todo: replace with our own middleware
	r.Use(slogchi.New(logger))
	r.Use(middleware.Recoverer)

	r.Use(cors.Handler(cors.Options{AllowedOrigins: cfg.CorsOrigins}))

	r.Mount("/v0", v0_routes.GetRoutes(cfg))

	http.ListenAndServe(":8080", r)
}
