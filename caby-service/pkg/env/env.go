package env

import (
	"fmt"
	"os"
	"strconv"
)

const (
	MissingErr     = `env '%s' is required`
	InvalidIntErr  = `env '%s' must be an integer: '%s'`
	InvalidBoolErr = `env '%s' must be a boolean (true/false): '%s'`
)

type Value interface {
	string | int | bool
}

type converter[T Value] func(string, string) (T, error)

func StringValue(key string, val string) (string, error) {
	return val, nil
}

func BoolValue(key string, val string) (bool, error) {
	b, err := strconv.ParseBool(val)
	if err != nil {
		err = fmt.Errorf(InvalidBoolErr, key, val)
	}
	return b, err
}

func IntValue(key string, val string) (int, error) {
	i, err := strconv.Atoi(val)
	if err != nil {
		err = fmt.Errorf(InvalidIntErr, key, val)
	}
	return i, err
}

func GetEnv[T Value](key string, conv converter[T]) (T, error) {
	value := os.Getenv(key)
	if value == "" {
		var t T
		return t, fmt.Errorf(MissingErr, key)
	}

	return conv(key, value)
}

func GetEnvOrDefault[T Value](key string, fallback T, conv converter[T]) (T, error) {
	value := os.Getenv(key)
	if value == "" {
		return fallback, nil
	}

	return conv(key, value)
}
