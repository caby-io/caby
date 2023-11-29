package env

import (
	"fmt"
	"log/slog"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
)

func ErrAttr(err error) slog.Attr {
	return slog.Any("error", err)
}

func TestErrors(t *testing.T) {
	os.Setenv("TESTING_VAL_A", "HELLO")
	_, err := GetEnv[bool]("TESTING_VAL_A", BoolValue)
	assert.EqualError(t, err, fmt.Sprintf(InvalidBoolErr, "TESTING_VAL_A", "HELLO"))
}
