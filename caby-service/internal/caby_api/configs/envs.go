package config

import (
	"caby-service/pkg/config"
	"caby-service/pkg/env"
	"log"
)

// All environment variables should be defined and described here

const DATA_PATH = "DATA_PATH"

func mutateConfigEnv(oldConfig config.Config) config.Config {
	b := config.NewBuilder().WithConfig(oldConfig)

	dataPath, ok, err := env.GetOptionalEnv[string](DATA_PATH, env.StringValue)
	if err != nil {
		log.Fatalf(err.Error())
	}
	if ok {
		b.WithDataPath(dataPath)
	}

	return b.Compile()
}
