package users

import (
	"fmt"
	"net/http"
)

type Handler struct{}

func NewHandler() Handler {
	return Handler{}
}

func (h Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPatch, http.MethodPut:
		w.WriteHeader(404)
		fmt.Fprintf(w, "Route not found")
	case http.MethodGet:
		userId := r.PathValue("userId")
		if userId == "" {
			w.WriteHeader(404)
			fmt.Fprintf(w, "Route not found")
		} else {
			HandleShow(userId, w, r)
		}
	case http.MethodPost:
		HandleRegister(w, r)
	case http.MethodDelete:
		HandleUnregister(w, r)
	}
}
