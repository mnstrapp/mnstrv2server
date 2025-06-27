package collect

import (
	"log"
	"net/http"
)

type Handler struct{}

func NewHandler() Handler {
	return Handler{}
}

func (h Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodGet, http.MethodPatch, http.MethodPut:
		w.WriteHeader(404)
		log.Printf("Route not found: %s", r.URL.Path)
	case http.MethodPost:
		HandleCollect(w, r)
	}
}