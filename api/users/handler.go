package users

import (
	"fmt"
	"log"
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
		log.Printf("userId: %s", userId)
		if userId == "" {
			w.WriteHeader(404)
			fmt.Fprintf(w, "Route not found")
		} else {
			HandleShow(userId, w, r)
		}
	case http.MethodPost:
		userId := r.PathValue("userId")
		if userId == "" {
			w.WriteHeader(404)
			fmt.Fprintf(w, "Route not found")
			return
		}
		HandleRegister(w, r)
	case http.MethodDelete:
		userId := r.URL.Query().Get("userId")
		if userId != "" {
			w.WriteHeader(404)
			fmt.Fprintf(w, "Route not found")
			return
		}
		HandleUnregister(w, r)
	}
}
