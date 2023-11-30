package env

import (
	"errors"
	"log/slog"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
)

func ErrAttr(err error) slog.Attr {
	return slog.Any("error", err)
}

func TestErrors(t *testing.T) {
	_, err := GetEnv[bool]("MISSING_ENV_VAR", BoolValue)
	assert.True(t, errors.As(err, &MissingErr{}))

	os.Setenv("BAD_BOOL_VAR", "hello")
	_, err = GetEnv[bool]("BAD_BOOL_VAR", BoolValue)
	assert.True(t, errors.As(err, &InvalidBoolErr{}))

	os.Setenv("BAD_INT_VAR", "abc")
	_, err = GetEnv[int]("BAD_INT_VAR", IntValue)
	assert.True(t, errors.As(err, &InvalidIntErr{}))
}
