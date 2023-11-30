package config

import "caby-service/pkg/config"

type MutateConfig func(config.Config) config.Config

// todo: add more layers: yaml config file
// LoadConfig generates config.Config from a heirchy:
//  1. Defaults
//  2. Environment variables
//
// The compiled config, as opposed to a builder, is passed between layers so we can add context at the layer of failure
func LoadConfig() config.Config {
	cfg := config.NewBuilder().WithDataPath("/srv").Compile()

	cfg = mutateConfigEnv(cfg)

	// todo: validate config

	return cfg
}
