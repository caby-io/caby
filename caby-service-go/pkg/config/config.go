package config

type Config struct {
	ServerAddr  string   // IP/Port that the http server binds to
	DataPath    string   // The root path of the files we are serving
	CorsOrigins []string // Allowed origins
}

type ConfigBuilder func(Config) Config

func (b ConfigBuilder) WithConfig(config Config) ConfigBuilder {
	return func(c Config) Config {
		return config
	}
}

func (b ConfigBuilder) WithDataPath(path string) ConfigBuilder {
	return func(c Config) Config {
		cfg := b(c)
		cfg.DataPath = path
		return cfg
	}
}

func (b ConfigBuilder) WithCorsOrigins(origins []string) ConfigBuilder {
	return func(c Config) Config {
		cfg := b(c)
		cfg.CorsOrigins = origins
		return cfg
	}
}

func (b ConfigBuilder) Compile() Config {
	return b(Config{})
}

func NewBuilder() ConfigBuilder {
	return func(c Config) Config {
		return Config{}
	}
}
