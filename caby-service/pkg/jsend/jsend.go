package jsend

import (
	"encoding/json"
	"log/slog"
	"net/http"
)

const (
	STATUS_SUCCESS = "success"
	STATUS_FAIL    = "fail"
	STATUS_ERROR   = "error"
)

type JSend struct {
	HTTPStatus uint   `json:"-"`
	Status     string `json:"status"`
	Data       any    `json:"data,omitempty"`
	Message    any    `json:"message,omitempty"`
}

type JSendBuilder func(j JSend) JSend

func (b JSendBuilder) Ok() JSendBuilder {
	return func(j JSend) JSend {
		j = b(j)
		j.HTTPStatus = http.StatusOK
		j.Status = STATUS_SUCCESS
		j.Message = nil
		return j
	}
}

func (b JSendBuilder) Data(d any) JSendBuilder {
	return func(j JSend) JSend {
		j = b(j)
		j.Data = d
		return j
	}
}

func (b JSendBuilder) Write(w http.ResponseWriter) {
	j := b(JSend{})
	bytes, err := json.Marshal(j)
	if err != nil {
		slog.Error("couldn't marshal JSend response", "json.Marshal.error", err, "data", j.Data)
		w.WriteHeader(http.StatusInternalServerError)
		w.Write([]byte("internal server error"))
		return
	}
	w.Write(bytes)
}

func New() JSendBuilder {
	return func(c JSend) JSend {
		return JSend{}
	}
}
