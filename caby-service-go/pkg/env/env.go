package env

import (
	"errors"
	"fmt"
	"os"
	"strconv"
	"strings"
)

type MissingErr struct {
	Key string
}

func (e MissingErr) Error() string {
	return fmt.Sprintf("missing env: %s", e.Key)
}

type InvalidIntErr struct {
	Key   string
	Value string
}

func (e InvalidIntErr) Error() string {
	return fmt.Sprintf("env '%s' must be an integer, got '%s'", e.Key, e.Value)
}

type InvalidBoolErr struct {
	Key   string
	Value string
}

func (e InvalidBoolErr) Error() string {
	return fmt.Sprintf("env '%s' must be a boolean (true/false), got '%s'", e.Key, e.Value)
}

type Value interface {
	string | int | bool | []string
}

type converter[T Value] func(string, string) (T, error)

func StringValue(key string, val string) (string, error) {
	return val, nil
}

func BoolValue(key string, val string) (bool, error) {
	b, err := strconv.ParseBool(val)
	if err != nil {
		err = fmt.Errorf("%w: %s", InvalidBoolErr{key, val}, err)
	}
	return b, err
}

func IntValue(key string, val string) (int, error) {
	i, err := strconv.Atoi(val)
	if err != nil {
		err = fmt.Errorf("%w: %s", InvalidIntErr{key, val}, err)
	}
	return i, err
}

func StringSliceValue(key string, val string) ([]string, error) {
	split := strings.Split(val, ",")
	res := []string{}
	for _, s := range split {
		res = append(res, strings.TrimSpace(s))
	}
	return res, nil
}

func GetEnv[T Value](key string, conv converter[T]) (T, error) {
	value := os.Getenv(key)
	if value == "" {
		var t T
		return t, MissingErr{key}
	}

	return conv(key, value)
}

func GetEnvOrDefault[T Value](key string, fallback T, conv converter[T]) (T, error) {
	v, err := GetEnv[T](key, conv)
	if errors.Is(err, MissingErr{}) {
		return fallback, nil
	}

	return v, err
}

func GetOptionalEnv[T Value](key string, conv converter[T]) (T, bool, error) {
	v, err := GetEnv[T](key, conv)
	if errors.Is(err, MissingErr{}) {
		var t T
		return t, false, err
	}

	return v, true, nil
}
