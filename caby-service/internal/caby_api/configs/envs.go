package configs

import (
	"caby-service/pkg/config"
	"caby-service/pkg/env"
	"log"
)

const (
	ENV_SERVER_ADDRESS = "SERVER_ADDRESS"
	ENV_DATA_PATH      = "DATA_PATH"
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

func mutateConfigEnv(oldConfig config.Config) config.Config {
	b := config.NewBuilder().WithConfig(oldConfig)

	// todo: make a helper function to set or fail
	dataPath, ok, err := env.GetOptionalEnv[string](ENV_DATA_PATH, env.StringValue)
	b = setOrFail[string](dataPath, ok, err, b, b.WithDataPath)

	return b.Compile()
}
