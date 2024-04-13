package config

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestConfigBuilder(t *testing.T) {
	c := NewBuilder().WithDataPath("/var/storage").Compile()
	assert.Equal(t, "/var/storage", c.DataPath)
}

func TestExistingConfigBuilder(t *testing.T) {
	cb := NewBuilder().WithConfig(Config{DataPath: "/var/storage-old"})
	assert.Equal(t, "/var/storage-old", cb.Compile().DataPath)
	c := cb.WithDataPath("/var/storage-new").Compile()
	fmt.Println(c)
	assert.Equal(t, "/var/storage-new", c.DataPath)
}

func TestBuilderOrder(t *testing.T) {
	b := NewBuilder().WithDataPath("/srv/a").WithDataPath("/srv/b")
	assert.Equal(t, "/srv/b", b.Compile().DataPath)

	b = b.WithDataPath("/srv/c")
	assert.Equal(t, "/srv/c", b.Compile().DataPath)
}
