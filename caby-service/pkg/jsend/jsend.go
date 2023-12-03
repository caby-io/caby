package jsend

import (
	"encoding/json"
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
	Data       any    `json:"data"`
	Message    any    `json:"message"`
}

type JSendBuilder func(j JSend) JSend

func (b JSendBuilder) Ok() JSendBuilder {
	return func(j JSend) JSend {
		j = b(j)
		j.HTTPStatus = http.StatusOK
		j.Status = STATUS_SUCCESS
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
		//todo
		return
	}
	w.Write(bytes)
}

func New() JSendBuilder {
	return func(c JSend) JSend {
		return JSend{}
	}
}
