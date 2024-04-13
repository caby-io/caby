package configs

import (
	"caby-service/pkg/config"
	"caby-service/pkg/env"
	"log"
	"log/slog"
)

const (
	ENV_SERVER_ADDRESS = "SERVER_ADDRESS"
	ENV_DATA_PATH      = "DATA_PATH"
	ENV_CORS_ORIGINS   = "CORS_ORIGINS"
)

// todo: improve
func setOrFail[T env.Value](val T, ok bool, err error, builder config.ConfigBuilder, setter func(T) config.ConfigBuilder) config.ConfigBuilder {
	if err != nil {
		log.Fatalf(err.Error())
	}
	if ok {
		return setter(val)
	}
	return builder
}

// todo: improve
func setOrDefault[T env.Value](val T, ok bool, err error, setter func(T) config.ConfigBuilder, fallback T) config.ConfigBuilder {
	if err != nil {
		slog.Error("could not fetch env var: " + err.Error())
		return setter(fallback)
	}
	if ok {
		return setter(val)
	}
	return setter(fallback)
}

func mutateConfigEnv(oldConfig config.Config) config.Config {
	b := config.NewBuilder().WithConfig(oldConfig)

	// todo: make a helper function to set or fail
	dataPath, ok, err := env.GetOptionalEnv(ENV_DATA_PATH, env.StringValue)
	b = setOrFail(dataPath, ok, err, b, b.WithDataPath)

	corsOrigins, ok, err := env.GetOptionalEnv(ENV_CORS_ORIGINS, env.StringSliceValue)
	b = setOrDefault(corsOrigins, ok, err, b.WithCorsOrigins, []string{})

	return b.Compile()
}
